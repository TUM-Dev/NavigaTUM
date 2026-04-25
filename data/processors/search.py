import polars as pl

RANKING_FACTOR_BY_TYPE = {
    "root": 0,  # Not searchable
    "site": 1100,
    "campus": 1100,
    "area": 1100,
    "joined_building": 1100,
    "building": 1000,  # joined buildings take precedence over buildings
    "room": 100,
    "virtual_room": 200,
}
RANKING_FACTOR_BY_DIN_USAGE = {  # DIN-Desc in brackets
    "NF1.2": 100,  # Sozialraum, Kindergarten (Gemeinschaftsräume)
    "NF1.5": 100,  # Speiseraum, Cafeteria, Mensa (Speiseräume)
    "NF2.1": 100,  # Sekretariat (Büroräume)
    "NF2.3": 100,  # Sitzungszimmer, Besprechungsraum (Besprechungsräume)
    "NF2.8": 200,  # EDV-Raum (Bürotechnikräume)
    "NF3.4": 200,  # Labor - Physik, Video (Physikalische ... Labors)
    "NF3.5": 200,  # Labor - Chemie (Chemische ... Labors)
    "NF3.8": 100,  # Küche, Teeküche (Küchen)
    "NF4.4": 50,  # Poststelle, Anlieferung (Annahme- und Ausgaberäume)
    "NF5.1": 900,  # Hörsaal (Unterrichtsräume mit festem Gestühl)
    "NF5.2": 500,  # Seminarraum, Zeichensaal, Übungsraum (Allg. Unterrichtsräume ...)
    "NF5.3": 250,  # Musikunterricht (Besondere Unterrichtsräume ...)
    "NF5.4": 400,  # Lesesaal, Freihandbibliothek (Bibliotheksräume)
    "NF5.5": 150,  # Sportraum, Turnsaal, Schwimmhalle (Sporträume)
    "NF7.1": 100,  # WC (Sanitärräume)
    "NF7.3": 20,  # Fahrradraum (Abstellräume)
    "VF9.1": 2,  # Schleuse (Flure, Hallen)
    "VF9.2": 1,  # Treppenhaus (Treppen)
    "VF9.3": 1,  # Aufzugsschacht (Schächte für Förderanlagen)
    # Usages not listed here are not important
}


def add_ranking_base(lf: pl.LazyFrame) -> pl.LazyFrame:
    """
    Add the base ranking attributes by type and usage.

    Returns a LazyFrame with ranking_rank_type, ranking_rank_usage,
    and ranking_rank_boost columns added.
    """
    lf = lf.with_columns(
        [
            pl.col("type")
            .replace_strict(RANKING_FACTOR_BY_TYPE, default=100)
            .cast(pl.Int64)
            .alias("ranking_rank_type"),
            pl.when(pl.col("type") == "room")
            .then(pl.col("usage_din_277").replace_strict(RANKING_FACTOR_BY_DIN_USAGE, default=10).cast(pl.Int64))
            .otherwise(pl.lit(100))
            .alias("ranking_rank_usage"),
        ]
    )

    # Type-specific boosts
    lf = lf.with_columns(
        pl.when((pl.col("type") == "room") & pl.col("props_stats_n_seats").is_not_null())
        .then((pl.col("props_stats_n_seats") // 10).clip(0, 99))
        .when(pl.col("type").is_in(["building", "joined_building"]) & pl.col("props_stats_n_rooms_reg").is_not_null())
        .then((pl.col("props_stats_n_rooms_reg") // 20).clip(0, 99))
        .when(pl.col("type").is_in(["campus", "area", "site"]) & pl.col("props_stats_n_buildings").is_not_null())
        .then(pl.col("props_stats_n_buildings").clip(0, 99))
        .otherwise(pl.lit(None))
        .cast(pl.Int64)
        .alias("ranking_rank_boost"),
    )

    return lf


def add_ranking_combined(lf: pl.LazyFrame) -> pl.LazyFrame:
    """
    Add the combined ranking factor.

    Returns a LazyFrame with a ranking_rank_combined column added.
    """
    lf = lf.with_columns(
        pl.when(pl.col("ranking_rank_type").is_not_null())
        .then(
            (pl.col("ranking_rank_type") * pl.col("ranking_rank_usage")) // 100
            + pl.col("ranking_rank_boost").fill_null(0)
            + pl.col("ranking_rank_custom").fill_null(0)
        )
        .otherwise(pl.lit(10))
        .cast(pl.Int64)
        .alias("ranking_rank_combined"),
    )
    return lf
