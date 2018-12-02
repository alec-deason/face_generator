import sys
import re
import shutil
import os
from pathlib import Path

from lxml import etree

source = sys.argv[1]
destination = Path(sys.argv[2])

with open(source) as f:
    source = etree.parse(f)

def pallet_setup(root):
    titles = root.xpath(f"//svg:title[contains(text(), 'pallet_')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})
    colors = {}
    for t in titles:
        swatch = t.getparent()
        style = swatch.attrib["style"]
        color = re.search("fill:([^;]+)", style).group(1)
        colors[t.text[7:]] = color
    for e in root.iter():
        if "style" in e.attrib:
            for s, c in colors.items():
                e.attrib["style"] = e.attrib["style"].replace(c, s)

def save_all(root, object_type, destination):
    try:
        shutil.rmtree(destination)
    except FileNotFoundError:
        pass
    os.makedirs(destination, exist_ok=True)
    layers = root.xpath(f"//svg:g[contains(@inkscape:label, '{object_type}_')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})
    assets = [
            (layer.attrib['{http://www.inkscape.org/namespaces/inkscape}label'].partition("_")[-1],
             layer)
            for layer in layers
    ]
    for (asset_id, asset) in assets:
        asset.attrib["style"] = "display:inline"
        with open(destination / f"{asset_id}.svg", "wb") as f:
            f.write(etree.tostring(asset))

pallet_setup(source)

save_all(source, "face", destination / "face")
save_all(source, "ears", destination / "ears")
save_all(source, "nose", destination / "nose")
save_all(source, "mouth", destination / "mouth")
save_all(source, "eyes", destination / "eyes")
save_all(source, "eyebrows", destination / "eyebrows")
save_all(source, "hair", destination / "hair")
