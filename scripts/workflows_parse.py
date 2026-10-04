#!/usr/bin/env python3
"""Every workflow is valid YAML and names no key twice."""

import pathlib
import sys

try:
    import yaml
except ImportError:
    print("pyyaml is not here, so the workflows were not parsed")
    print("install it with: python3 -m pip install pyyaml")
    sys.exit(1)

MERGE = "tag:yaml.org,2002:merge"
UNDER = pathlib.Path(".github")


class Strict(yaml.SafeLoader):
    pass


def no_twice(loader, node, deep=False):
    seen = set()
    # compared as written: a 1.1 loader reads «on» as True and would name a key no file wrote
    for key, _ in node.value:
        if key.tag == MERGE or not isinstance(key, yaml.ScalarNode):
            continue
        if key.value in seen:
            raise yaml.YAMLError(f"the «{key.value}» key is repeated")
        seen.add(key.value)
    # merged after the comparison, like SafeConstructor: a key overriding an anchor's is legal
    loader.flatten_mapping(node)
    return yaml.SafeLoader.construct_mapping(loader, node, deep)


Strict.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, no_twice)

found = sorted(
    one
    for one in UNDER.rglob("*")
    if one.is_file() and one.suffix in (".yml", ".yaml")
)
if not found:
    print(f"no .yml or .yaml lives under {UNDER}, so nothing was parsed")
    sys.exit(1)

bad = 0
for one in found:
    try:
        yaml.load(one.read_text(encoding="utf-8"), Strict)
    except yaml.YAMLError as why:
        print(f"{one} is not valid YAML, so what it launches will not launch")
        print(why)
        bad = 1

sys.exit(bad)
