import { atom } from "recoil"

export const pocketPathAtom = atom<string | null>({
  key: "pocketPathAtom",
  default: null,
})

export const fileSystemInvalidationAtom = atom<number>({
  key: "fileSystemInvalidationAtom",
  default: Date.now(),
})

export const PocketModelColourAtom = atom<"black" | "white">({
  key: "PocketModelColourAtom",
  default: "black",
})

export const iventoryInvalidationAtom = atom<number>({
  key: "iventoryInvalidationAtom",
  default: Date.now(),
})

export const configInvalidationAtom = atom<number>({
  key: "configInvalidationAtom",
  default: Date.now(),
})
