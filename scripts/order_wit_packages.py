#!/usr/bin/env python3

import re
import sys
from pathlib import Path
from graphlib import TopologicalSorter, CycleError

# --- Configuration ---
WIT_ROOT = Path("wit")
# Define directories/patterns containing WIT packages
# Assumes main WIT file is named like the directory (e.g., wit/cli/cli.wit)
TARGET_DIRS = [WIT_ROOT / "cli"] + \
    [d for d in (WIT_ROOT / "extensions").iterdir() if d.is_dir()]
# --- End Configuration ---

# Regular expressions to parse WIT directives
# Captures the base package ID (namespace:name), ignoring version/interface/world details
PACKAGE_RE = re.compile(r"^\s*package\s+([a-zA-Z0-9:-]+)(?:@[\w.-]+)?\s*;")
IMPORT_RE = re.compile(r"^\s*import\s+([a-zA-Z0-9:-]+)(?:@[\w.-]+)?/.*?;")
INCLUDE_RE = re.compile(r"^\s*include\s+([a-zA-Z0-9:-]+)(?:@[\w.-]+)?/.*?;")


def find_main_wit_file(dir_path: Path) -> Path | None:
    """Finds the main .wit file in a directory, assuming it matches the dir name."""
    expected_file = dir_path / f"{dir_path.name}.wit"
    if expected_file.is_file():
        return expected_file
    # Fallback: look for any .wit file if the named one isn't found
    wit_files = list(dir_path.glob("*.wit"))
    if len(wit_files) == 1:
        print(
            f"Warning: Using '{wit_files[0].name}' as main WIT file for '{dir_path}', expected '{expected_file.name}'.", file=sys.stderr)
        return wit_files[0]
    elif len(wit_files) > 1:
        print(
            f"Error: Multiple .wit files found in '{dir_path}'. Cannot determine main file.", file=sys.stderr)
        return None
    else:
        print(f"Error: No .wit file found in '{dir_path}'.", file=sys.stderr)
        return None


def build_dependency_graph(target_dirs: list[Path]) -> tuple[dict[str, set[str]], dict[str, Path]]:
    """
    Builds a dependency graph from WIT files.

    Returns:
        tuple: (graph, package_to_dir_map)
               graph: Dict mapping base_package_id to a set of its base_dependency_ids.
               package_to_dir_map: Dict mapping base_package_id to its directory Path.
    """
    graph: dict[str, set[str]] = {}
    package_to_dir: dict[str, Path] = {}
    errors_found = False

    for dir_path in target_dirs:
        if not dir_path.is_dir():
            print(
                f"Warning: Skipping non-directory path '{dir_path}'", file=sys.stderr)
            continue

        wit_file = find_main_wit_file(dir_path)
        if not wit_file:
            errors_found = True
            continue

        try:
            content = wit_file.read_text()
            base_package_id = None
            dependencies = set()

            for line in content.splitlines():
                package_match = PACKAGE_RE.match(line)
                if package_match:
                    base_package_id = package_match.group(1)
                    continue  # Process package line first

                import_match = IMPORT_RE.match(line)
                if import_match:
                    dependencies.add(import_match.group(1))
                    continue

                include_match = INCLUDE_RE.match(line)
                if include_match:
                    dependencies.add(include_match.group(1))
                    continue

            if base_package_id:
                if base_package_id in package_to_dir:
                    print(
                        f"Error: Duplicate package ID '{base_package_id}' defined in '{dir_path}' and '{package_to_dir[base_package_id]}'.", file=sys.stderr)
                    errors_found = True
                else:
                    package_to_dir[base_package_id] = dir_path
                    graph[base_package_id] = dependencies
            else:
                print(
                    f"Error: No 'package ...;' directive found in '{wit_file}'.", file=sys.stderr)
                errors_found = True

        except Exception as e:
            print(f"Error processing file '{wit_file}': {e}", file=sys.stderr)
            errors_found = True

    if errors_found:
        print("\nErrors encountered during graph construction. Aborting.",
              file=sys.stderr)
        sys.exit(1)

    # Ensure all dependencies mentioned exist as nodes in the graph
    all_defined_packages = set(graph.keys())
    all_dependencies = set(dep for deps in graph.values() for dep in deps)
    missing_dependencies = all_dependencies - all_defined_packages
    if missing_dependencies:
        print("Error: The following dependencies are imported/included but not defined by any scanned package:", file=sys.stderr)
        for missing in sorted(list(missing_dependencies)):
            print(f"- {missing}", file=sys.stderr)
        print("\nAborting.", file=sys.stderr)
        sys.exit(1)

    return graph, package_to_dir


def main():
    print("Building WIT dependency graph...", file=sys.stderr)
    graph, package_to_dir = build_dependency_graph(TARGET_DIRS)

    print("Performing topological sort...", file=sys.stderr)
    try:
        ts = TopologicalSorter(graph)
        ordered_packages = list(ts.static_order())
    except CycleError as e:
        print("\nError: A cycle was detected in the WIT package dependencies:", file=sys.stderr)
        print(f"Cycle details: {e.args[1]}", file=sys.stderr)
        print("Please resolve the circular dependencies.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\nError during topological sort: {e}", file=sys.stderr)
        sys.exit(1)

    print("Dependency order determined successfully.", file=sys.stderr)
    # Print the ordered directory paths
    for pkg_id in ordered_packages:
        print(package_to_dir[pkg_id])


if __name__ == "__main__":
    main()
