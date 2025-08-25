import logging
from math import prod
from zipfile import ZipFile
import polars as pl
import datetime as dt

from external.scraping_utils import _download_file, CACHE_PATH
from utils import setup_logging

HST_REPORT_URL = "https://www.opendata-oepnv.de/fileadmin/datasets/delfi/20250310_zHV_gesamt.zip"
OPEN_DATA_OEPNV_URL = "https://www.opendata-oepnv.de/ht/de/organisation/delfi/startseite?tx_vrrkit_view%5Baction%5D=details&tx_vrrkit_view%5Bcontroller%5D=View&tx_vrrkit_view%5Bdataset_name%5D=deutschlandweite-haltestellendaten&cHash=02aa95607cd0164111fcf703f749a2ee"
PUBLIC_TRANSPORT_CACHE_PATH = CACHE_PATH / "public_transport"


def _load_stations() -> pl.DataFrame:
    """Load the stations from the HST_REPORT data and add them to stations dict"""
    zip_file_path = PUBLIC_TRANSPORT_CACHE_PATH / "gesamt.zip"
    # _download_file(HST_REPORT_URL, zip_file_path)
    if not zip_file_path.exists():
        raise ValueError(f"""File {zip_file_path} not found.
                        due to weird reasons they don't allow downloading the file, except in a browser.
                        => please get this file manually and put it in the cache folder under the name gesamt.zip
                        """)
    with ZipFile(zip_file_path) as file_zip:
        files = [f for f in file_zip.namelist() if f.endswith(".csv")]
        assert len(files) == 1, f"Expected 1 CSV file, but found {len(files)}: {files}"
        file_name = files[0]
        file_zip.extract(file_name, PUBLIC_TRANSPORT_CACHE_PATH)
    logging.info(f"Extracted the zip file to {file_name}")
    df = pl.read_csv(PUBLIC_TRANSPORT_CACHE_PATH / file_name, separator=";", decimal_comma=True)

    # datatype cleanup
    df = df.with_columns(
        pl.col("LastOperationDate")
        .str.to_datetime(
            "%Y-%m-%dT%H:%M:%S",
            strict=False,  # the rest are empty collumns => cast to null
        )
        .cast(pl.Date),
    )
    df = df.with_columns(pl.when(pl.col("SEV") == "ja").then(True).otherwise(False).name.keep())
    with pl.StringCache():
        df = df.with_columns(pl.col("Authority").cast(pl.Categorical).name.keep())
    # P: "boarding_postion
    # Q: "quay <- "Mast"
    # A: "area" <- interesing, but does not have DELFI routing data
    df = df.filter(~pl.col("Type").is_in(["Q", "P", "A"]))
    df.drop_in_place("Type")
    # df = df.with_columns(
    #    pl.col("Type")
    #    .map_elements(lambda x: {"S": "stop", "A": "area"}[x], return_dtype=pl.String)
    #    .cast(type_enum)
    #    .name.keep()
    # )
    df = df.with_columns(
        pl.when(pl.col("DelfiName").is_in(["-", "", None]))
        .then(pl.col("Name"))
        .otherwise(pl.col("DelfiName"))
        .name.map(lambda x: "Name")
    )
    df.drop_in_place("DelfiName")
    df = df.with_columns(pl.col("Latitude").cast(pl.Float32).name.keep())
    df = df.with_columns(pl.col("Longitude").cast(pl.Float32).name.keep())

    # geo bounds from https://gist.github.com/graydon/11198540
    valid_coordinates = df["Latitude"].map_elements(
        lambda x: x is not None and 47.2701114 < x < 55.099161, return_dtype=pl.Boolean
    )
    valid_coordinates &= df["Longitude"].map_elements(
        lambda x: x is not None and 5.8663153 < x < 15.0419319, return_dtype=pl.Boolean
    )
    valid_coordinates &= ~df["Longitude"].is_nan()
    valid_coordinates &= ~df["Latitude"].is_nan()
    invalid_coordinates = ~valid_coordinates
    if invalid_coordinates.any():
        logging.warning(f"Dropped {invalid_coordinates.sum()} / {df['DHID'].count()} rows due to invalid coordinates")
        df = df.filter(valid_coordinates)

    if df["DHID"].is_null().any():
        logging.warning(f"Dropped {df['DHID'].is_null().sum()} / {df['DHID'].count()} rows due to missing DHID")
        df = df.filter(~(df["DHID"].is_null()))

    if df["Name"].is_null().any():
        logging.warning(f"Dropped {df['Name'].is_null().sum()} / {df['Name'].count()} rows due to missing Name")
        df = df.filter(~(df["Name"].is_null()))

    last_operation_date = df.drop_in_place("LastOperationDate")
    had_last_operation_date = ~last_operation_date.is_null() & (last_operation_date < dt.datetime.now())
    if had_last_operation_date.any():
        logging.warning(
            f"Dropped {had_last_operation_date.sum()} / {df['DHID'].count()} rows due to having had the LastOperationDate"
        )
        df = df.filter(~had_last_operation_date)

    # if parent = dhid, this is not very usefull information ^^
    df = df.with_columns(pl.when(pl.col("Parent") != pl.col("DHID")).then(pl.col("Parent")).otherwise(None).name.keep())

    df = df.drop(
        [
            "SeqNo",
            "THID",
            "TariffProvider",
            "Description",
            "MunicipalityCode",
            "Municipality",
            "DistrictCode",
            "District",
            "Authority",
        ],
        strict=True,
    )
    df = df.rename(
        {
            "DHID": "dhid",
            "Parent": "parent",
            "Name": "name",
            "Latitude": "lat",
            "Longitude": "lon",
            "SEV": "is_sev",
        },
        strict=True,
    )
    return df


def scrape_stations() -> None:
    """Scrape the stations from the MVV GTFS data and return them as a list of dicts"""
    logging.info("Scraping the bus and train stations of the MVV")
    df = _load_stations()
    df.write_parquet(CACHE_PATH / "public_transport.parquet", use_pyarrow=True, compression_level=22)


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    PUBLIC_TRANSPORT_CACHE_PATH.mkdir(exist_ok=True)
    scrape_stations()
