import ifcopenshell
import multiprocessing
import ifcopenshell.geom

ifc_filepath = "/home/frank/dev/tum-dev/Navigatum/map/data/5505_IFC4/02-55-5505-100_EG.ifc"
ifc_file = ifcopenshell.open(ifc_filepath)

settings = ifcopenshell.geom.settings()
iterator = ifcopenshell.geom.iterator(settings, ifc_file, multiprocessing.cpu_count())
if iterator.initialize():
    while True:
        shape = iterator.get()
        element = ifc_file.by_id(shape.id)
        matrix = shape.transformation.matrix
        faces = shape.geometry.faces
        edges = shape.geometry.edges
        verts = shape.geometry.verts

        if not iterator.next():
            break
