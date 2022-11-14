import { platform } from "os"
import { useRecoilValue } from "recoil"
import {
  CoreAuthorImageSelectorFamily,
  CoreInfoSelectorFamily,
  CoreInventorySelector,
} from "../../../recoil/selectors"
import { Controls } from "../../controls"
import { Link } from "../../link"

import "./index.css"
import { PlatformImage } from "../platformImage"
import { useInventoryItem } from "../../../hooks/useInventoryItem"
import { Releases } from "./releases"
import { Version } from "../version"

type CoreInfoProps = {
  coreName: string
  onBack: () => void
}

export const CoreInfo = ({ coreName, onBack }: CoreInfoProps) => {
  const coreInfo = useRecoilValue(CoreInfoSelectorFamily(coreName))
  const authorImageSrc = useRecoilValue(CoreAuthorImageSelectorFamily(coreName))
  const inventoryItem = useInventoryItem(coreName)

  return (
    <div className="core-info">
      <Controls
        controls={[
          {
            type: "back-button",
            text: "Back to list",
            onClick: onBack,
          },
        ]}
      />

      <h3 className="core-info__title">{coreInfo.core.metadata.shortname}</h3>
      {coreInfo.core.metadata.platform_ids.map((platformId) => (
        <PlatformImage
          className="core-info__image"
          platformId={platformId}
          key={platformId}
        />
      ))}

      <section className="core-info__info">
        <p>{coreInfo.core.metadata.description}</p>

        <div className="core-info__info-row">
          <strong>{"Version:"}</strong>
          <Version coreName={coreName} />
        </div>

        <div className="core-info__info-row">
          <strong>{"Author:"}</strong>
          <div className="core-info__author-tag">
            <img src={authorImageSrc} />
            {coreInfo.core.metadata.author}
          </div>
        </div>

        {coreInfo.core.metadata.url && (
          <div className="core-info__info-row">
            <strong>{"URL:"}</strong>

            <Link href={coreInfo.core.metadata.url}>
              {coreInfo.core.metadata.url}
            </Link>
          </div>
        )}
        {coreInfo.core.metadata.date_release && (
          <div className="core-info__info-row">
            <strong>{"Release Date:"}</strong>

            {coreInfo.core.metadata.date_release}
          </div>
        )}

        <div className="core-info__info-row">
          <strong>{"Supports Sleep:"}</strong>
          <input
            readOnly
            type="checkbox"
            checked={coreInfo.core.framework.sleep_supported}
          />
        </div>

        <div className="core-info__info-row">
          <strong>{"Supports Dock:"}</strong>
          <input
            readOnly
            type="checkbox"
            checked={coreInfo.core.framework.dock.supported}
          />
        </div>

        <div className="core-info__info-row">
          <strong>{"Supports Dock Analog:"}</strong>
          <input
            readOnly
            type="checkbox"
            checked={coreInfo.core.framework.dock.analog_output}
          />
        </div>

        <div className="core-info__info-row">
          <strong>{"Supports Cartridges:"}</strong>
          <input
            readOnly
            type="checkbox"
            checked={coreInfo.core.framework.hardware.cartridge_adapter !== -1}
          />
        </div>
        {inventoryItem && inventoryItem.repository.platform === "github" && (
          <Releases inventoryItem={inventoryItem} />
        )}
      </section>
    </div>
  )
}
