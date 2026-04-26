from external.models import public_transport


def test_public_transport():
    """Load all stations from the public_transport.Station"""
    public_transport.Station.load_all()
