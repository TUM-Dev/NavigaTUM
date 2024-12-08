# pip install ifcopenshell
# pip install mathutils

import ifcopenshell
import ifcopenshell.draw
import ifcopenshell.api.context
import ifcopenshell.util.shape
from ifcopenshell import geom
import plotly.express as px
import pandas as pd

ifc_file = "/home/frank/dev/tum-dev/Navigatum/map/data/5505_IFC4/02-55-5505-100_EG.ifc"

model = ifcopenshell.open(ifc_file)
rooms = model.by_type('IfcSpace')
# doors = model.by_type('IfcDoor')
vertecies = []
for i, room in enumerate(rooms):
    settings = ifcopenshell.geom.settings()
    try:
        shape = ifcopenshell.geom.create_shape(settings, room)
    except Exception:
        continue
    # Since the lists are flattened, you may prefer to group them like so depending on your geometry kernel
    # A nested numpy array e.g. [[v1x, v1y, v1z], [v2x, v2y, v2z], ...]
    grouped_verts = ifcopenshell.util.shape.get_vertices(shape.geometry)
    # A nested numpy array e.g. [[e1v1, e1v2], [e2v1, e2v2], ...]
    # grouped_edges = ifcopenshell.util.shape.get_edges(shape.geometry)
    # A nested numpy array e.g. [[f1v1, f1v2, f1v3], [f2v1, f2v2, f2v3], ...]
    # grouped_faces = ifcopenshell.util.shape.get_faces(shape.geometry)
    ifcopenshell.util.geom
    matrix = shape.transformation.matrix
    (raw_x, raw_y, raw_z) = matrix[12:15]
    # print(location)
    vertecies.extend(
        dict(x=raw_x + x, y=raw_y + y, z=raw_z + z, id=i) for (x, y, z) in grouped_verts if (raw_z + z) < 2)
    if i > 2:
        continue

df = pd.DataFrame(vertecies)
fig = px.line(df, x="x", y="y", color="id")
fig.update_yaxes(scaleanchor="x", scaleratio=1, overwrite=True)
fig.show()
# plan = ifcopenshell.api.context.add_context(model, context_type="Plan")
settings = ifcopenshell.draw.draw_settings()
ifcopenshell.draw.main(settings, [model])
