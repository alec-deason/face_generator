import sys
import re
import shutil
import os
from pathlib import Path
import json

from lxml import etree

source = Path(sys.argv[1])
feature_name = source.stem
destination = Path(sys.argv[2]) / feature_name
try:
        shutil.rmtree(destination)
except FileNotFoundError:
        pass
os.makedirs(destination, exist_ok=True)

with open(source) as f:
    source = etree.parse(f)

guide_layers = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and contains(@inkscape:label, 'guide')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})

def process_side(guide_layer, side):
    layers = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and re:match(@inkscape:label, '[0-9]+{side}')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape", "re":"http://exslt.org/regular-expressions"})
    layers = [(e.attrib["{http://www.inkscape.org/namespaces/inkscape}label"], e) for e in layers]



    def process_guide(guide):
        if guide.tag == "{http://www.w3.org/2000/svg}rect":
            x = float(guide.attrib["x"])
            y = float(guide.attrib["y"])
            xx = float(guide.attrib["width"]) + x
            yy = float(guide.attrib["height"]) + y
            return (
                x, y, xx, y, xx, yy, x, yy,
            )
        else:
            return (
                float(guide.attrib["cx"]),
                float(guide.attrib["cy"]),
                float(guide.attrib["r"]),
                None, None, None, None, None,
            )

    guide = process_guide(list(guide_layer.iterchildren())[0])

    def process_feature(feature, destination, name, guide):
        with open(destination / f"{name}.json", "w") as f:
            json.dump(guide, f)
        with open(destination / f"{name}.svg", "wb") as f:
            f.write(etree.tostring(feature))

    for (name, layer) in layers:
        process_feature(layer, destination, name, guide)

if len(guide_layers) > 1:
    guide_layer = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and contains(@inkscape:label, 'guide_left')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]
    process_side(guide_layer, "_left")
    guide_layer = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and contains(@inkscape:label, 'guide_right')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]
    process_side(guide_layer, "_right")
else:
    process_side(guide_layers[0], "")
