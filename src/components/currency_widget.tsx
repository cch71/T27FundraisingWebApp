import React from 'react'
import currency from "currency.js"

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

const CurrencyWidget = ({defaultValue, label, onInput, id }) => {

    function formatCurrency(evt) {
        evt.currentTarget.value = USD(evt.currentTarget.value).format();
    }

    return (
        <div className="form-floating">
            <input type="text" min="0" step="any" className="form-control"
                   pattern="^\$\d{1,3}(,\d{3})*(\.\d+)?$"
                   data-type="currency"
                   id={id}
                   defaultValue={defaultValue}
                   placeholder="$0.00"
                   onBlur={formatCurrency}
                   onInput={onInput}
            />
            <label htmlFor={id}>{label}</label>
        </div>
    );
}

export default CurrencyWidget;
