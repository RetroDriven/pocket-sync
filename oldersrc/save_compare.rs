use crate::{
    cores::TransformCore,
    pocket_files::{convert_rom_path_to_save_path, find_roms_for_save},
    PlatformSave, SaveInfo,
};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
pub struct SavePair<'a> {
    pocket: &'a SaveInfo,
    mister: &'a SaveInfo,
}

impl SavePair<'_> {
    pub fn is_pocket_newer(&self) -> bool {
        self.pocket.date_modified > self.mister.date_modified
    }

    pub fn newer_save(&self) -> &SaveInfo {
        if self.pocket.date_modified > self.mister.date_modified {
            self.pocket
        } else {
            self.mister
        }
    }

    pub fn older_save(&self) -> &SaveInfo {
        if self.pocket.date_modified > self.mister.date_modified {
            self.mister
        } else {
            self.pocket
        }
    }
}

impl<'a> fmt::Display for SavePair<'a> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let titles = match self.is_pocket_newer() {
            true => ("-- Pocket (newer)", "-- MiSTer (older)"),
            false => ("-- MiSTer (newer)", "-- Pocket (older)"),
        };

        write!(
            f,
            "{}\n{} \n\n--- VS ---\n\n{}\n{}",
            titles.0,
            self.newer_save(),
            titles.1,
            self.older_save()
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum SaveComparison<'a> {
    PocketOnly(&'a SaveInfo),
    MiSTerOnly(&'a SaveInfo),
    PocketNewer(SavePair<'a>),
    MiSTerNewer(SavePair<'a>),
    Conflict(SavePair<'a>),
    NoSyncNeeded,
}

impl SaveComparison<'_> {
    pub fn use_mister(
        &self,
        ftp_stream: &mut suppaftp::FtpStream,
        pocket_path: &PathBuf,
    ) -> Result<(), suppaftp::FtpError> {
        let mister_save_info = match self {
            SaveComparison::MiSTerOnly(save_info) => &save_info,
            Self::PocketNewer(save_pair)
            | Self::MiSTerNewer(save_pair)
            | Self::Conflict(save_pair) => &save_pair.mister,
            _ => {
                panic!("Attempt to use a non-existent MiSTer save");
            }
        };
        let path = &mister_save_info.path;
        let _ = ftp_stream.cwd(path.parent().unwrap().to_path_buf().to_str().unwrap())?;
        let file_name = path.file_name().unwrap();
        let mut save_file = ftp_stream.retr_as_buffer(file_name.to_str().unwrap())?;

        let pocket_save_paths: Vec<PathBuf> = match self {
            SaveComparison::MiSTerOnly(save_info) => {
                let found = find_roms_for_save(
                    &save_info.game,
                    &save_info.core.rom_filetypes(),
                    &pocket_path,
                )
                .iter()
                .map(|p| convert_rom_path_to_save_path(p))
                .collect();

                found
            }
            Self::PocketNewer(save_pair)
            | Self::MiSTerNewer(save_pair)
            | Self::Conflict(save_pair) => vec![pocket_path.join(save_pair.pocket.path.clone())],
            Self::PocketOnly(_) => panic!("Attempt to use a non-existent MiSTer save"),
            Self::NoSyncNeeded => panic!("Attempt to sync when NoSyncNeeded"),
        };

        if pocket_save_paths.len() == 0 {
            println!(
                "Couldn't find \"{}\" on the pocket, skipping",
                mister_save_info.game.replace(".sav", "")
            );
            return Ok(());
        }

        println!(
            "Copying {} ({}) \nMiSTer -> Pocket",
            mister_save_info.game,
            mister_save_info.core.to_pocket()
        );

        for pocket_save_path in pocket_save_paths {
            let prefix = pocket_save_path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            let mut file = File::create(pocket_save_path).unwrap();
            let mut buf: Vec<u8> = Vec::new();
            save_file.read_to_end(&mut buf).unwrap();
            file.write(&buf).unwrap();
        }

        return Ok(());
    }

    pub fn use_pocket(
        &self,
        ftp_stream: &mut suppaftp::FtpStream,
        pocket_path: &PathBuf,
    ) -> Result<(), suppaftp::FtpError> {
        let pocket_save_info = match self {
            SaveComparison::PocketOnly(save_info) => &save_info,
            Self::PocketNewer(save_pair)
            | Self::MiSTerNewer(save_pair)
            | Self::Conflict(save_pair) => &save_pair.pocket,
            _ => {
                panic!("Attempt to use a non-existent Pocket save");
            }
        };
        let path = &pocket_save_info.path;
        let file_name = path.file_name().unwrap();
        let mister_save_path = match self {
            SaveComparison::PocketOnly(save_info) => {
                pocket_path.join(format!("/media/fat/saves/{}", save_info.core.to_mister()))
            }
            Self::PocketNewer(save_pair)
            | Self::MiSTerNewer(save_pair)
            | Self::Conflict(save_pair) => save_pair
                .mister
                .path
                .clone()
                .parent()
                .unwrap()
                .to_path_buf(),
            Self::MiSTerOnly(_) => panic!("Attempt to use a non-existent MiSTer save"),
            Self::NoSyncNeeded => panic!("Attempt to sync when NoSyncNeeded"),
        };

        let mut file = File::open(path).unwrap();

        let mister_path_buf = &mister_save_path.to_path_buf();
        let mister_path = mister_path_buf.to_str().unwrap();

        println!(
            "Copying {} ({}) \nPocket -> MiSTer\n---",
            pocket_save_info.game,
            pocket_save_info.core.to_mister()
        );

        ftp_stream.cwd(mister_path)?;
        ftp_stream.put_file(file_name.to_str().unwrap(), &mut file)?;

        return Ok(());
    }
}

pub fn check_save<'a>(
    save: &'a PlatformSave,
    pocket_saves: &'a Vec<PlatformSave>,
    mister_saves: &'a Vec<PlatformSave>,
    last_merge: i64,
) -> SaveComparison<'a> {
    match save {
        PlatformSave::PocketSave(pocket_save_info) => {
            if let Some(mister_save_info) =
                find_matching_mister_save(pocket_save_info, mister_saves)
            {
                return get_comparison(pocket_save_info, mister_save_info, last_merge);
            } else {
                return SaveComparison::PocketOnly(pocket_save_info);
            }
        }
        PlatformSave::MiSTerSave(mister_save_info) => {
            if let Some(pocket_save_info) =
                find_matching_pocket_save(mister_save_info, pocket_saves)
            {
                return get_comparison(pocket_save_info, mister_save_info, last_merge);
            } else {
                return SaveComparison::MiSTerOnly(mister_save_info);
            }
        }
    }
}

fn get_comparison<'a>(
    pocket_save_info: &'a SaveInfo,
    mister_save_info: &'a SaveInfo,
    last_merge: i64,
) -> SaveComparison<'a> {
    if mister_save_info.date_modified < 86400 {
        // MiSTer save was updated while the RTC wasn't running - raise as a conflict to be safe
        return SaveComparison::Conflict(SavePair {
            pocket: pocket_save_info,
            mister: mister_save_info,
        });
    }

    if pocket_save_info.date_modified < last_merge && mister_save_info.date_modified < last_merge {
        return SaveComparison::NoSyncNeeded;
    }

    if pocket_save_info.date_modified > last_merge && mister_save_info.date_modified > last_merge {
        return SaveComparison::Conflict(SavePair {
            pocket: pocket_save_info,
            mister: mister_save_info,
        });
    }

    if mister_save_info.date_modified > pocket_save_info.date_modified {
        return SaveComparison::MiSTerNewer(SavePair {
            pocket: pocket_save_info,
            mister: &mister_save_info,
        });
    } else {
        return SaveComparison::PocketNewer(SavePair {
            pocket: pocket_save_info,
            mister: mister_save_info,
        });
    }
}

fn find_matching_mister_save<'a>(
    save: &SaveInfo,
    saves: &'a Vec<PlatformSave>,
) -> Option<&'a SaveInfo> {
    for mister_save in saves {
        if let PlatformSave::MiSTerSave(mister_save) = mister_save {
            if mister_save.core == save.core && mister_save.game == save.game {
                return Some(&mister_save);
            }
        }
    }
    return None;
}

fn find_matching_pocket_save<'a>(
    save: &SaveInfo,
    saves: &'a Vec<PlatformSave>,
) -> Option<&'a SaveInfo> {
    for pocket_save in saves {
        if let PlatformSave::PocketSave(pocket_save) = pocket_save {
            if pocket_save.core == save.core && pocket_save.game == save.game {
                return Some(&pocket_save);
            }
        }
    }
    return None;
}

pub fn remove_duplicates<'a>(save_comparisons: Vec<SaveComparison<'a>>) -> Vec<SaveComparison<'a>> {
    let mut singles: Vec<SaveComparison> = Vec::new();

    for save_comparison in save_comparisons {
        match &save_comparison {
            SaveComparison::NoSyncNeeded => singles.push(save_comparison),
            _ => {
                if !singles.contains(&save_comparison) {
                    singles.push(save_comparison)
                }
            }
        }
    }

    singles
}
