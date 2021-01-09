import currency from "currency.js"
import {orderDb, Order} from "../js/ordersdb"

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

////////////////////////////////////////////////////////
// Save off current order values
const saveCurrentOrder = ()=>{
    const currentOrder = orderDb.getActiveOrder();
    if (!(document.getElementById('newOrEditOrderForm') && currentOrder)) { return; }

    console.log("Saving Order");
    //Required
    currentOrder.orderOwner = (document.getElementById('formOrderOwner') as HTMLInputElement).value;
    currentOrder.firstName = (document.getElementById('formFirstName') as HTMLInputElement).value;
    currentOrder.lastName = (document.getElementById('formLastName') as HTMLInputElement).value;
    currentOrder.phone = (document.getElementById('formPhone') as HTMLInputElement).value;
    currentOrder.addr1 = (document.getElementById('formAddr1') as HTMLInputElement).value;
    currentOrder.neighborhood = (document.getElementById('formNeighborhood') as HTMLSelectElement).value;


    currentOrder.email = (document.getElementById('formEmail') as HTMLInputElement).value;
    currentOrder.addr2 = (document.getElementById('formAddr2') as HTMLInputElement).value;
    /* currentOrder.city = (document.getElementById('formCity') as HTMLInputElement).value;
     * currentOrder.state = (document.getElementById('formState') as HTMLInputElement).value;
     * currentOrder.zip = (document.getElementById('formZip') as HTMLInputElement).value;
     */
    currentOrder.specialInstructions =
        (document.getElementById('formSpecialInstructions') as HTMLInputElement).value;
    currentOrder.checkNums = (document.getElementById('formCheckNumbers') as HTMLInputElement).value;
    currentOrder.cashPaid = currency((document.getElementById('formCashPaid') as HTMLInputElement).value);
    currentOrder.checkPaid = currency((document.getElementById('formCheckPaid') as HTMLInputElement).value);
    currentOrder.doCollectMoneyLater  = (document.getElementById('formCollectLater') as HTMLInputElement).checked;
    currentOrder.totalAmt = currency(currentOrder.donation).add(currency(currentOrder.productsCost));
    console.log(`Current Order ${JSON.stringify(currentOrder, null, 2)}`);
}

export {onCurrencyFieldKeyPress, onCheckNumsKeyPress, onNonNumsKeyPress, moneyFloor, saveCurrentOrder}
