import dataframely as dy
import polars as pl

# Value sets for the Roomfinder Enums. The Roomfinder dataset is frozen upstream so these lists
# act both as documentation and as a strict drift gate at load time.
_BUILDINGS_B_AREA = [
    "Dachau",
    "Eichenau",
    "Eichenau - Staatsgut Roggenstein",
    "Freising - Veitshof",
    "Freising - WZW Berg",
    "Freising - WZW Campus",
    "Freising - WZW Dürnast",
    "Freising - WZW Externe",
    "Freising - WZW Hochfeld",
    "Freising - WZW Sonstige",
    "Garching - Betriebsgebäude I",
    "Garching - Chemie",
    "Garching - Elektrotechnik",
    "Garching - Forschungs-Reaktor",
    "Garching - Garching sonst.",
    "Garching - IMETUM",
    "Garching - MW",
    "Garching - Math/Inf.",
    "Garching - Meteo-Mast, Oskar-von-Miller Turm",
    "Garching - Physik",
    "Garching - Werkfeuerwehr TUM Garching",
    "Garching - Zentraler Bereich",
    "Iffeldorf - Limnologische Station",
    "Kranzberg - Staatsgut Thalhausen",
    "München - Biederstein",
    "München - Großhadern",
    "München - Kapuzinerhölzl",
    "München - Klinikum Rechts der Isar",
    "München - München Sonst.",
    "München - Pasing",
    "München - Stammgelände Nord",
    "München - Stammgelände Süd",
    "München - Stammgelände Südost",
    "München - Stammgelände Südwest",
    "München - Stammgelände Zentral",
    "München - TUM Campus im Olympiapark",
    "München - Winzererstraße",
    "Nationalpark Berchtesgaden",
    "Obernach/Walchensee",
    "Obernach/Weilheim - Obernach",
    "Ottobrunn",
    "Raitenhaslach",
    "Sonstiges",
    "Starnberg",
    "Straubing",
    "WZW Helmholtz-Zentrum",
]
_R_LEVEL = ["-", "-1", "-2", "0", "0.", "0zg", "1", "1zg", "2", "2zg", "3", "4", "5", "6", "7", "DG", "ZG"]
_DEFAULT_MAP_SCALE = ["500", "1000", "1200", "2000", "4000", "5000", "10000", "200000", "400000", "500000"]
_DEFAULT_MAP_ID = [
    "rf10", "rf12", "rf142", "rf156", "rf160", "rf161", "rf162", "rf168", "rf172", "rf173", "rf174",
    "rf175", "rf176", "rf177", "rf178", "rf179", "rf180", "rf181", "rf182", "rf187", "rf192", "rf201",
    "rf248", "rf54", "rf80",
]
_DEFAULT_MAP_NAME = [
    "FMI Übersicht",
    "Garching b. München 11-13 Lageplan",
    "Garching b. München 35-39 Lageplan",
    "Iffeldorf",
    "Karlstraße 45-47 Lageplan",
    "Lageplan Campus Garching",
    "Lageplan TUM",
    "Marsstr. 20-22 Lageplan",
    "München",
    "München und Umgebung",
    "Olympiapark ZHS",
    "Physik I Garching 1.OG",
    "Physik I Garching 1.UG",
    "Physik I Garching 2.OG",
    "Physik I Garching 2.UG",
    "Physik I Garching EG",
    "Physik II Garching 1.OG",
    "Physik II Garching EG",
    "Physik II Garching UG",
    "Stammgelände Basiskarte",
    "Uptown München",
    "WZ Weihenstephan Mitte",
    "WZ Weihenstephan Süd",
    "WZ Weihenstephan Übersicht",
    "Wissenschaftszentrum Straubing Lageplan",
]


class BuildingsSchema(dy.Schema):
    """Schema for the Roomfinder building catalogue (`buildings_roomfinder.csv`)."""

    b_alias = dy.String(nullable=False)
    b_area = dy.Enum(_BUILDINGS_B_AREA, nullable=False)
    b_id = dy.String(nullable=False)
    b_name = dy.String(nullable=False)
    b_room_count = dy.Int64(nullable=False)
    lat = dy.Float64(nullable=False)
    lon = dy.Float64(nullable=False)
    default_map_scale = dy.Enum(_DEFAULT_MAP_SCALE, nullable=True)
    default_map_id = dy.Enum(_DEFAULT_MAP_ID, nullable=True)
    default_map_name = dy.Enum(_DEFAULT_MAP_NAME, nullable=True)
    default_map_width = dy.Int64(nullable=True)
    default_map_height = dy.Int64(nullable=True)
    maps_json = dy.String(nullable=False)


class RoomsSchema(dy.Schema):
    """Schema for the Roomfinder room catalogue (`rooms_roomfinder.csv`)."""

    lat = dy.Float64(nullable=False)
    lon = dy.Float64(nullable=False)
    r_alias = dy.String(nullable=False)
    r_id = dy.String(nullable=False)
    r_level = dy.Enum(_R_LEVEL, nullable=False)
    r_number = dy.String(nullable=False)
    b_alias = dy.String(nullable=False)
    b_area = dy.Enum(_BUILDINGS_B_AREA, nullable=False)
    b_id = dy.String(nullable=False)
    b_name = dy.String(nullable=False)
    default_map_scale = dy.Enum(_DEFAULT_MAP_SCALE, nullable=False)
    default_map_id = dy.Enum(_DEFAULT_MAP_ID, nullable=False)
    default_map_name = dy.Enum(_DEFAULT_MAP_NAME, nullable=False)
    default_map_width = dy.Int64(nullable=False)
    default_map_height = dy.Int64(nullable=False)
    maps_json = dy.String(nullable=False)
    metas_json = dy.String(nullable=False)


class MapsSchema(dy.Schema):
    """Schema for the Roomfinder map catalogue (`maps_roomfinder.csv`)."""

    id = dy.String(nullable=False)
    desc = dy.String(nullable=False)
    height = dy.Int64(nullable=False)
    width = dy.Int64(nullable=False)
    scale = dy.Enum(_DEFAULT_MAP_SCALE, nullable=False)
    latlonbox_north = dy.Float64(nullable=False)
    latlonbox_south = dy.Float64(nullable=False)
    latlonbox_east = dy.Float64(nullable=False)
    latlonbox_west = dy.Float64(nullable=False)
    latlonbox_rotation = dy.Float64(nullable=False)
    file = dy.String(nullable=False)
    source = dy.Enum(["Roomfinder"], nullable=False)
