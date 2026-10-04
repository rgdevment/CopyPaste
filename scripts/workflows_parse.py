#!/usr/bin/env python3
"""Every workflow is valid YAML and names no key twice."""

import pathlib
import sys

try:
    import yaml
except ImportError:
    # rules.sh reads an exit of 0 as «this rule passed» and throws the output away, so a rule
    # that cannot look has to fail. Install it with: python3 -m pip install pyyaml
    print("pyyaml is not here, so the workflows were not parsed")
    sys.exit(1)


class Strict(yaml.SafeLoader):
    pass


def no_twice(loader, node, deep=False):
    # a merge key is resolved into the mapping before the keys are read, the way SafeConstructor
    # does it; without this an anchor merged with << is reported as an unparseable document
    loader.flatten_mapping(node)
    seen = set()
    for key, _ in node.value:
        name = loader.construct_object(key, deep=deep)
        if name in seen:
            raise yaml.YAMLError(f"the «{name}» key is repeated")
        seen.add(name)
    return yaml.SafeLoader.construct_mapping(loader, node, deep)


Strict.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, no_twice)

bad = 0
for one in sorted(pathlib.Path(".github").rglob("*.yml")):
    try:
        yaml.load(one.read_text(encoding="utf-8"), Strict)
    except yaml.YAMLError as why:
        print(f"{one} is not valid YAML, so what it launches will not launch")
        print(why)
        bad = 1

sys.exit(bad)
