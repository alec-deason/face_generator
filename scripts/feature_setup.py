import sys
import re
import shutil
import os
from pathlib import Path
import json

from lxml import etree
from svg.path import parse_path

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
        elif guide.tag == "{http://www.w3.org/2000/svg}path":
            d = guide.attrib["d"]
            path = parse_path(d)

            path = [(p.start.real, p.start.imag) for p in path]
            assert len(path) == 4

            return (
                path[0][0], path[0][1], path[1][0], path[1][1],
                path[2][0], path[2][1], path[3][0], path[3][1],
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
        with open(destination / f"{name}.svg", "w") as f:
            f.write("<svg viewBox='0 0 210 210' xmlns:svg='http://www.w3.org/2000/svg' xmlns='http://www.w3.org/2000/svg'><!-- PALLETE -->"+etree.tostring(feature).decode("utf8")+"</svg>")

    for (name, layer) in layers:
        process_feature(layer, destination, name, guide)

if len(guide_layers) > 1:
    guide_layer = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and contains(@inkscape:label, 'guide_left')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]
    process_side(guide_layer, "_left")
    guide_layer = source.xpath(f"//svg:g[@inkscape:groupmode = 'layer' and contains(@inkscape:label, 'guide_right')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]
    process_side(guide_layer, "_right")
else:
    process_side(guide_layers[0], "")
