import currency from "currency.js"
import awsConfig from "../config"

interface Product<T> {
    label: string,
    costDescription: string,
    cost: T,
}

interface ProductCurrency {
    label: string,
    costDescription: string,
    cost: currency,
}

interface DeliveryDate {
    date: string;
    disabledDate: string;
}

interface FundraiserConfigBase<T> {
    kind: string;
    description: string;
    products: Record<string, Product<T>>;
    neighborhoods: Array<string>;
    deliveryDates: Array<DeliveryDate>;
}


class FundraiserConfig {
    readonly kind: string;
    readonly description: string;
    readonly products: Record<string, ProductCurrency>;
    readonly neighborhoods: Array<string>;
    readonly deliveryDates: Array<DeliveryDate>

    constructor(dlFrConfig: FundraiserConfigBase<string>|null) {

        const getCurrencyProdMap = (oldProd: Record<string, Product<string>>): Record<string, ProductCurrency> => {

            let newProdMap: Record<string, ProductCurrency> = {};
            for (let entry of Object.entries(oldProd)) {
                const productId: string = entry[0];
                const oldProd: Product<string> = entry[1];

                const newProd: ProductCurrency = {
                    cost: currency(oldProd.cost),
                    label: oldProd.label,
                    costDescription: oldProd.costDescription
                };
                console.log(`Add Config: ${productId}: ${JSON.stringify(newProd)}`);
                newProdMap[productId] = newProd; 
            };

            console.log(`Old Prod Map: ${JSON.stringify(newProdMap)}`);
            return(newProdMap);
        };


        const getConfig = (): FundraiserConfigBase<string> => {
            if (null === dlFrConfig) {
                let sessionFrConfig = window.sessionStorage.getItem('frConfig');
                if (sessionFrConfig) {
                    console.error('Loading from session');
                    return JSON.parse(sessionFrConfig);
                } else {
                    console.error("Failed to load Session Fr Config");
                    throw ("Failed to load Session Fundraiser Config");
                }
            } else {
                return dlFrConfig;
            }
        };

        

        const loadedFrConfig: FundraiserConfigBase<string> = getConfig();

        this.kind = loadedFrConfig.kind;
        this.products = getCurrencyProdMap(loadedFrConfig.products);
        this.description = loadedFrConfig.description;
        this.neighborhoods = loadedFrConfig.neighborhoods;
        this.deliveryDates = loadedFrConfig.deliveryDates;
    }

    /////////////////////////////////////////
    //
    *validDeliveryDates(): IterableIterator<string> {
        
        for (let deliveryDate of this.deliveryDates) {
            //if delivery date < disabledDate
            yield deliveryDate.date;
        }
        yield 'donation';
    }

}


let frConfig: FundraiserConfig|null = null;

function downloadFundraiserConfig(authToken: string): Promise<FundraiserConfig | null> {
    return new Promise(async (resolve, reject)=>{
        try {
            const resp = await fetch(awsConfig.api.invokeUrl + '/getconfig', {
                method: 'post',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: authToken
                },
                body: JSON.stringify({})
            });

            if (!resp.ok) { // if HTTP-status is 200-299
                alert("HTTP Resp Error: " + resp.status);
                reject(null);
            } else {
                const loadedFrConfig: FundraiserConfigBase<string> = await resp.json();
                console.log(`Fundraiser Config: ${JSON.stringify(loadedFrConfig)}`);

                window.sessionStorage.setItem('frConfig', JSON.stringify(loadedFrConfig));
                frConfig = new FundraiserConfig(loadedFrConfig);
                resolve(frConfig);
            }
        } catch(error) {
            console.error(error);
            alert("HTTP-Error: " + error);
            reject(null);
        }
    });
}

function getFundraiserConfig(): FundraiserConfig {
    if (null===frConfig) {
        return new FundraiserConfig(null);
    }
    return frConfig;
}


export { FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig};
