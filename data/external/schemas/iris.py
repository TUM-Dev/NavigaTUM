import dataframely as dy


class IrisRoomsSchema(dy.Schema):
    """AStA Iris learning-room roster (`iris.csv`)."""

    # The `<arch_name>@<building_id>` form, joined against NavigaTUM aliases. Globally unique.
    raum_nr_architekt = dy.String(nullable=False, primary_key=True)
    # The NavigaTUM building id (verified 1:1), used as a cross-check on the alias join.
    gebaeude_code = dy.String(nullable=False)
