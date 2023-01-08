
/////////////////////////////////////////////////////////////////////
//
const createSellMap = (params) => {
    // new rust code converts it to map so have to convert it back to
    // JSON Object (or TODO:re-write all this)
    //const params = Object.fromEntries(mapOfParams);

    console.log("Creating Sell Map Report View");
    // JSON.stringify(params, ' ', '\t'));

    const map = L.map(params.id).setView(params.centerPt, 12);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        // maxZoom: 19,
        attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    }).addTo(map);

    if (params.geoJson) {
        const geojsonMarkerOptions = {
            radius: 8,
            fillColor: "#ff7800",
            color: "#000",
            weight: 1,
            opacity: 1,
            fillOpacity: 0.8
        };

        L.geoJSON(params.geoJson, {
            pointToLayer: function (feature, latlng) {
                return L.circleMarker(latlng, geojsonMarkerOptions);
            }
        }).addTo(map);
    }

    return map;

};

export {createSellMap}
