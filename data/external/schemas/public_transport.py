import dataframely as dy

# Mirrors `ModeResponse` in `server/src/routes/maps/route/motis.rs`.
TRANSPORT_MODES: list[str] = [
    "walk",
    "bike",
    "rental",
    "car",
    "car_parking",
    "car_dropoff",
    "odm",
    "flex",
    "transit",
    "tram",
    "subway",
    "suburban",
    "ferry",
    "airplane",
    "metro",
    "bus",
    "coach",
    "rail",
    "highspeed_rail",
    "long_distance",
    "night_rail",
    "regional_fast_rail",
    "regional_rail",
    "cable_car",
    "funicular",
    "areal_lift",
    "ride_sharing",
    "other",
]


class StationsSchema(dy.Schema):
    """Public-transport station catalogue (`public_transport.parquet`)."""

    id = dy.String(nullable=False, primary_key=True)
    name = dy.String(nullable=False)
    modes = dy.List(dy.Enum(TRANSPORT_MODES, nullable=False), nullable=False, min_length=1)
    lat = dy.Float64(nullable=False)
    lon = dy.Float64(nullable=False)
