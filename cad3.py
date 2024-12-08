# pip install ifcopenshell
# pip install mathutils

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.draw
import ifcopenshell.api.context
import ifcopenshell.util.shape
import plotly.express as px
import pandas as pd

ifc_file = "/home/frank/dev/tum-dev/Navigatum/map/data/5505_IFC4/02-55-5505-100_EG.ifc"

model = ifcopenshell.open(ifc_file)
rooms = model.by_type('IfcSpace')
# doors = model.by_type('IfcDoor')
setttings = ifcopenshell

settings = ifcopenshell.geom.settings()
settings.set(settings.USE_PYTHON_OPENCASCADE, True)

try:
    shape = ifcopenshell.geom.create_shape(settings, rooms[0])
    geometry = shape.geometry  # see #1124
    # These are methods of the TopoDS_Shape class from pythonOCC
    shape_gpXYZ = geometry.Location().Transformation().TranslationPart()
    # These are methods of the gpXYZ class from pythonOCC
    print(shape_gpXYZ.X(), shape_gpXYZ.Y(), shape_gpXYZ.Z())
except:
    print("Shape creation failed")
