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

const moneyFloor = (value: string)=>{
	let isChanged = false;
	if (!value) { return [currency(0), isChanged]; }
	let comps = value.split(".");
	if (1<comps.length && 2<comps[1].length) {
		comps[1]=comps[1].slice(0,2);
		isChanged = true;
	}
	comps = comps.join('.');
	//console.log(`${value}    ${comps}`);
	const amt = currency(Math.abs(parseFloat(comps)));
	//console.log(`${value}, ${amt.toString()}`);
	isChanged = isChanged || parseFloat(value)!==amt.value;
	return [amt, isChanged];
};

export {onCurrencyFieldKeyPress, onCheckNumsKeyPress, onNonNumsKeyPress, moneyFloor}
