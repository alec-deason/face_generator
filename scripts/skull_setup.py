import sys
import re
import shutil
import os
from pathlib import Path
import json

from lxml import etree
from svg.path import parse_path

source = sys.argv[1]
destination = Path(sys.argv[2]) / "skulls"

with open(source) as f:
    source = etree.parse(f)

templates = source.xpath(f"//svg:g[contains(@inkscape:label, 'skull_')]", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})
templates = [(e.attrib["{http://www.inkscape.org/namespaces/inkscape}label"].partition("_")[-1], e) for e in templates]

def get_component(template, name):
    component = template.xpath(f"./*[@inkscape:label = '{name}']", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]
    if component.tag == "{http://www.w3.org/2000/svg}rect":
        x = float(component.attrib["x"])
        y = float(component.attrib["y"])
        w = float(component.attrib["width"])
        h = float(component.attrib["height"])
        return (x, y, w, h)
    elif component.tag == "{http://www.w3.org/2000/svg}circle":
        cx = float(component.attrib["cx"])
        cy = float(component.attrib["cy"])
        r = float(component.attrib["r"])
        return ((cx, cy), r)
    else:
        raise Exception(f"unknown component type '{component.tag}'")

def get_outline(template):
    component = template.xpath(f"./svg:path[@inkscape:label = 'skull']", namespaces={"svg": "http://www.w3.org/2000/svg", "inkscape": "http://www.inkscape.org/namespaces/inkscape"})[0]

    d = component.attrib["d"]
    path = parse_path(d)

    path = [(p.start.real, p.start.imag) for p in path]
    return path

def process_template(template, destination):
    try:
            shutil.rmtree(destination)
    except FileNotFoundError:
            pass
    os.makedirs(destination, exist_ok=True)
    print(name)
    output = {}

    output["eyeball_left"] = get_component(template, "eyeball_left")
    output["eyeball_right"] = get_component(template, "eyeball_right")
    output["ear_left"] = get_component(template, "ear_left")
    output["ear_right"] = get_component(template, "ear_right")
    output["mouth"] = get_component(template, "mouth")
    output["nose"] = get_component(template, "nose")
    output["hair"] = get_component(template, "hair")
    output["outline"] = get_outline(template)
    print(output)

    
    with open(destination / "skull.json", "w") as f:
        json.dump(output, f)

for (name, template) in templates:
    process_template(template, destination / name)

