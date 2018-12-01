import sys
import shutil
import os
from pathlib import Path

from lxml import etree

source = sys.argv[1]
destination = Path(sys.argv[2])

with open(source) as f:
    source = etree.parse(f)

def save_all(root, object_type, destination):
    shutil.rmtree(destination)
    os.makedirs(destination, exist_ok=True)
    layers = root.xpath(f"//svg:g[contains(@inkscape:label, '{object_type}_')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})
    assets = [
            (layer.attrib['{http://www.inkscape.org/namespaces/inkscape}label'].rsplit("_")[-1],
             layer)
            for layer in layers
    ]
    for (asset_id, asset) in assets:
        asset.attrib["style"] = "display:inline"
        with open(destination / f"{asset_id}.svg", "wb") as f:
            f.write(etree.tostring(asset))

save_all(source, "face", destination / "face")
save_all(source, "ears", destination / "ears")
save_all(source, "nose", destination / "nose")
save_all(source, "mouth", destination / "mouth")
save_all(source, "eyes", destination / "eyes")
save_all(source, "hair", destination / "hair")
