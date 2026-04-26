import dataframely as dy
import polars as pl


class UsagesSchema(dy.Schema):
    """Schema for the TUMonline usage type catalogue (`usages_tumonline.csv`)."""

    usage_id = dy.Int64(primary_key=True, nullable=False)
    din277_id = dy.String(nullable=False)
    din277_name = dy.String(nullable=False)
    name = dy.String(nullable=False)

    @dy.rule()
    def usage_id_positive(cls) -> pl.Expr:
        return pl.col("usage_id") > 0


class OrgsSchema(dy.Schema):
    """Schema for the TUMonline organisation catalogue (`orgs-{lang}_tumonline.csv`)."""

    org_id = dy.Int64(primary_key=True, nullable=False)
    code = dy.String(nullable=False)
    name = dy.String(nullable=False)
    path = dy.String(nullable=False)

    @dy.rule()
    def org_id_positive(cls) -> pl.Expr:
        return pl.col("org_id") > 0


class BuildingsSchema(dy.Schema):
    """Schema for the TUMonline building catalogue (`buildings_tumonline.csv`)."""

    building_key = dy.String(primary_key=True, nullable=False)
    address_place = dy.String(nullable=False)
    address_street = dy.String(nullable=False)
    address_zip_code = dy.Int64(nullable=False)
    area_id = dy.Int64(nullable=False)
    name = dy.String(nullable=False)
    tumonline_id = dy.Int64(nullable=False)
    filter_id = dy.Int64(nullable=True)

    @dy.rule()
    def building_key_is_four_digits(cls) -> pl.Expr:
        return pl.col("building_key").str.contains(r"^\d{4}$")

    @dy.rule()
    def tumonline_id_positive(cls) -> pl.Expr:
        return pl.col("tumonline_id") > 0


class RoomsSchema(dy.Schema):
    """Schema for the TUMonline room catalogue (`rooms_tumonline.csv`)."""

    room_key = dy.String(primary_key=True, nullable=False)
    address_place = dy.String(nullable=False)
    address_street = dy.String(nullable=False)
    address_zip_code = dy.Int64(nullable=False)
    seats_sitting = dy.Int64(nullable=True)
    seats_wheelchair = dy.Int64(nullable=True)
    seats_standing = dy.Int64(nullable=True)
    floor_type = dy.String(nullable=False)
    floor_level = dy.String(nullable=False)
    tumonline_id = dy.Int64(nullable=False)
    area_id = dy.Int64(nullable=False)
    building_id = dy.Int64(nullable=False)
    main_operator_id = dy.Int64(nullable=False)
    usage_id = dy.Int64(nullable=False)
    alt_name = dy.String(nullable=True)
    arch_name = dy.String(nullable=True)
    calendar_resource_nr = dy.Int64(nullable=True)
    patched = dy.Bool(nullable=False)

    @dy.rule()
    def tumonline_id_positive(cls) -> pl.Expr:
        return pl.col("tumonline_id") > 0
