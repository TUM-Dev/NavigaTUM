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


def add_ranking_base(data: dict) -> None:
    """
    Add the base ranking attributes by type and usage

    This operates on the data dict directly without creating a copy.
    """
    for _data in data.values():
        ranking_factors = _data.setdefault("ranking_factors", {})

        ranking_factors["rank_type"] = RANKING_FACTOR_BY_TYPE.get(_data["type"], 100)

        din_usage: str = _data.get("usage", {}).get("din_277", "")
        ranking_factors["rank_usage"] = (
            RANKING_FACTOR_BY_DIN_USAGE.get(din_usage, 10) if _data["type"] == "room" else 100
        )
        # Type-specific boosts
        if stats := _data.get("props", {}).get("stats", None):
            rank_boost = None
            if _data["type"] == "room" and "n_seats" in stats:
                rank_boost = stats["n_seats"] // 10
            elif _data["type"] in {"building", "joined_building"} and "n_rooms_reg" in stats:
                rank_boost = stats["n_rooms_reg"] // 20
            elif _data["type"] in {"campus", "area", "site"} and "n_buildings" in stats:
                rank_boost = stats["n_buildings"]

            if rank_boost is not None:
                ranking_factors["rank_boost"] = min(rank_boost, 99)


def add_ranking_combined(data: dict) -> None:
    """
    Add the combined ranking factor.

    This operates on the data dict directly without creating a copy.
    """
    for _data in data.values():
        if "ranking_factors" in _data:
            factors = _data["ranking_factors"]
            type_usage_ranking = factors["rank_type"] * factors["rank_usage"]
            factors["rank_combined"] = (
                type_usage_ranking // 100 + factors.get("rank_boost", 0) + factors.get("rank_custom", 0)
            )

        else:
            _data["ranking_factors"] = {
                "rank_combined": 10,  # low rank
            }
