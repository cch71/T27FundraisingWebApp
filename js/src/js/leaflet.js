import L, {
  Map as LeafletMap,
  TileLayer,
  GeoJSON,
  CircleMarker,
} from "leaflet";

/////////////////////////////////////////////////////////////////////
//
const createSellMap = (params) => {
  console.log("Creating Sell Map Report View");
  // console.log(JSON.stringify(params.geoJson, ' ', '\t'));

  const map = new LeafletMap(params.id).setView(params.centerPt, 12);

  new TileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution:
      '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  }).addTo(map);

  if (params.geoJson) {
    const geojsonMarkerOptions = {
      radius: 8,
      fillColor: "#ff7800",
      color: "#000",
      weight: 1,
      opacity: 1,
      fillOpacity: 0.8,
    };

    new GeoJSON(params.geoJson, {
      pointToLayer: function (geoJsonPt, latlng) {
        return new CircleMarker(latlng, geojsonMarkerOptions);
      },
    }).addTo(map);
  }

  return map;
};

export { createSellMap };
