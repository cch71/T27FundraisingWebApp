import currency from "currency.js"

const onCurrencyFieldKeyPress = (evt: any)=>{
    const charCode = (evt.which) ? evt.which : event.keyCode;
    if (45 === charCode) {
        evt.preventDefault();
        evt.stopPropagation();
        return false;
    }
    if (charCode != 46 && charCode > 31 && (charCode < 48 || charCode > 57)) {
        evt.preventDefault();
        evt.stopPropagation();
        return false;
    }
    return true;
};

const onNonNumsKeyPress = (evt: any)=>{
    const charCode = (evt.which) ? evt.which : event.keyCode;
    if (46===charCode) { //'.'
        evt.preventDefault();
        evt.stopPropagation();
        return false;
    }
    return onCurrencyFieldKeyPress(evt);
};


const onCheckNumsKeyPress = (evt: any)=>{
    const charCode = (evt.which) ? evt.which : event.keyCode;
    if (44/*','*/ === charCode || 32/*' '*/ === charCode) { return true; }
    if (46===charCode) { //'.'
        evt.preventDefault();
        evt.stopPropagation();
        return false;
    }
    return onCurrencyFieldKeyPress(evt);
};



export {onCurrencyFieldKeyPress, onCheckNumsKeyPress, onNonNumsKeyPress}
