#!/usr/bin/env python3
"""Every workflow is valid YAML and names no key twice."""

import os
import pathlib
import sys

try:
    import yaml
except ImportError:
    # a rule that cannot look has to shout in CI and step aside on a machine, or it is a rule
    # somebody uninstalls
    print("pyyaml is not here, so the workflows were not parsed")
    sys.exit(1 if os.environ.get("GITHUB_ACTIONS") else 0)


class Strict(yaml.SafeLoader):
    pass


def no_twice(loader, node, deep=False):
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
