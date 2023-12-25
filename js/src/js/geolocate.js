
function getExactCurrentPosition() {
    return new Promise((resolve, reject) => 
                       navigator.geolocation.getCurrentPosition(
                           resolve, 
                           reject,
                           {timeout:10000, maximumAge:0, enableHighAccuracy:true})
    );
}

export {getExactCurrentPosition};
