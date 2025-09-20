import subprocess  # nosec
import shutil
from pathlib import Path
import re

# Directories
OVERLAYS_DIR = Path("./overlays")
LEVELS_DIR.mkdir(exist_ok=True)
LEVELS_DIR = Path("./overlay_levels")


def convert_geotiff_to_mbtiles(tif_path: Path):
    """Convert a GeoTIFF file to an MBTiles file."""
    mbtiles_path: Path = LEVELS_DIR / tif.name.replace("_modified.tif", ".mbtiles")
    print(f"Processing {tif_path} -> {mbtiles_path}")

    subprocess.run(
        [
            "rio",
            "mbtiles",
            "--format",
            "WEBP",
            "--zoom-levels",
            "16..20",
            "--resampling",
            "lanczos",
            "--rgba",
            "--progress-bar",
            "--exclude-empty-tiles",
            str(tif_path),
            str(mbtiles_path),
        ],
        check=True,
    )  # nosec


def process_level(level: str):
    """Process a level by converting GeoTIFF files to MBTiles files."""
    print(f"Processing level {level}")
    input_files = list(LEVELS_DIR.glob(f"*_{level}.mbtiles"))
    if not input_files:
        continue

    sources = ",".join([f.name.replace(".mbtiles", "") for f in input_files])

    output = LEVELS_DIR / f"{level}.mbtiles"

    subprocess.run(
        [
            "martin-cp",
            *map(str, input_files),
            "--source",
            sources,
            "--auto-bounds",
            "calc",
            "--encoding",
            "identity",
            "--max-zoom",
            "20",
            "--min-zoom",
            "16",
            "--mbtiles-type",
            "normalized",
            "--output-file",
            str(output),
        ],
        check=True,
        env={"RUST_LOG": "info", "PATH": str(Path().absolute())},
    )  # nosec

    for f in input_files:
        f.unlink()


if __name__ == "__main__":
    for tif in OVERLAYS_DIR.glob("*_modified.tif"):
        convert_geotiff_to_mbtiles(tif)

    # Extract level numbers from filenames (the suffix after "_")
    levels = sorted(
        {
            re.search(r"_(.+)\.mbtiles", f.name).group(1)
            for f in LEVELS_DIR.glob("*.mbtiles")
            if re.search(r"_(.+)\.mbtiles", f.name)
        }
    )
    for level in levels:
        process_level(level)
