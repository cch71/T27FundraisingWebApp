
/////////////////////////////////////////////////////////////////////
//
const getCommonDtOptions = (tableColumns) => {

    const extractASrcHrefAndAlias = (data) => {
        const href = data.match(/href=['"]([^'"]*)['"]/);
        const text = data.match(/<a[^>]*>([^<]*)<\/a>/); // if it's a <a> tag
        if (href && text) {
            return [href[1], text[1]];
        }
        return [data, null];
    };

    const htmlColFormatter = (data, row, column) => {
        const tableColumn = tableColumns[column];
        if (tableColumn.title === "Map") {
            const [href, text] = extractASrcHrefAndAlias(data)
            if (href && text) {
                return `=HYPERLINK("${href}", "${text}")`;
            }
        }
        return data;
    };

    const textColFormatter = (data, row, column) => {
        const tableColumn = tableColumns[column];
        if (tableColumn.title === "Map") {
            const [href, text] = extractASrcHrefAndAlias(data)
            if (href && text) {
                return `"${href}"`;
            }
        }
        return data;
    };

    return {
        scrollResize: true,
        scrollCollapse: true,
        paging: false,
        lengthChange: false,
        responsive: true,
        deferRender: true,
        buttons: [
        ],
        layout: {
            topStart: {
                buttons: [
                    {
                        extend: 'copy',
                        exportOptions: {
                            format: {
                                body: htmlColFormatter
                            }
                        }
                    }, {
                        extend: 'excel',
                        exportOptions: {
                            format: {
                                body: htmlColFormatter
                            }
                        }
                    }, {
                      extend: 'csv',
                          exportOptions: {
                          format: {
                              body: textColFormatter
                          }
                      }
                  }, "pdf", "print", 'colvis'
                ]
            },
            topEnd: {
                search: {
                    placeholder: 'Search'
                }
            }
        },
        columns: tableColumns
    };
}
/////////////////////////////////////////////////////////////////////
//
const getQuickViewReportDataTable = (params) => {
    console.log("Setting Quick Report View");
    const tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Delivery Date", name: "DeliveryDate", type: "string" },
        { title: "Spreaders", name: "Spreaders", visible: false },
        {
            title: "Spreading",
            type: "string",
            render: (data, _, row, meta) => {
                if (0 !== row[meta.col - 1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////////
//
const getMoneyCollectionReportDataTable = (params) => {
    console.log("Setting Money Collection Report View");
    const tableColumns = [
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Delivery Date", name: "DeliveryDate", type: "string" },
        { title: "Total From Checks", name: "TotalFromChecks" },
        { title: "Total From Cash", name: "TotalFromCash" },
        { title: "Total" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////
//
const getDeliveriesViewReportDataTable = (params) => {
    console.log("Setting Deliveries Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Delivery Date", name: "DeliveryDate", className: "all", type: "string" },
        { title: "Name", className: "all" },
        { title: "Neighborhood" },
        { title: "Address" },
        { title: "Map" },
        { title: "Bags", type: "string" },
        { title: "Phone", type: "string" },
        { title: "Location" },
        { title: "Notes" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
    ];
    const dtOpts = getCommonDtOptions(tableColumns);
    dtOpts["order"] = [[1, "asc"]];
    return new DataTable(params.id, dtOpts);
};

/////////////////////////////////////////////////////////////////
//
const getDistPointsViewReportDataTable = (params) => {
    console.log("Setting Distrtibution Points Report View");
    let tableColumns = [
        { title: "Delivery Date", name: "DeliveryDate", className: "all", type: "string" },
        { title: "Total Bags", name: "TotalBags", className: "all", type: "string" },
    ];

    for (const header of params.distPoints) {
        tableColumns.push({ title: header, className: "all" });
    }

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};



/////////////////////////////////////////////////////////////////////
//
const getFullViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Phone", type: "string" },
        { title: "Email" },
        { title: "Address 1" },
        { title: "Address 2" },
        { title: "City" },
        { title: "Zipcode" },
        { title: "Neighborhood" },
        { title: "Delivery Date", type: "string" },
        { title: "Spreaders", name: "Spreaders", visible: false },
        {
            title: "Spreading",
            type: "string",
            render: (data, _, row, meta) => {
                if (0 !== row[meta.col - 1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Bags", type: "string" },
        { title: "Special Instructions" },
        { title: "Donations" },
        { title: "Cash" },
        { title: "Check" },
        { title: "Check Numbers" },
        { title: "Total Amount" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Verified" },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////////
//
const getOrderVerificationViewReportDataTable = (params) => {
    console.log("Setting Order Verification Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Delivery Date", type: "string" },
        { title: "Donations" },
        { title: "Cash" },
        { title: "Check" },
        { title: "Check Numbers" },
        { title: "Total Amount" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Verified" },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////////
//
const getSpreadingJobsUnfinishedViewReportDataTable = (params) => {
    console.log("Setting Spreading Jobs Unfinished Report View");
    let tableColumns = [
        { title: "Order Owner", name: "OrderOwner" },
        { title: "Name" },
        { title: "Delivery Date", type: "string" },
        { title: "Bags Left To Spread", className: "all", type: "string" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////////
//
const getSpreadingJobsViewReportDataTable = (params) => {
    console.log("Setting Spreading Jobs Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Phone", className: "all", type: "string" },
        { title: "Delivery Date", type: "string" },
        { title: "Instructions" },
        { title: "Address" },
        { title: "Neighborhood", className: "all" },
        { title: "Spreaders", name: "Spreaders", visible: false },
        {
            title: "Spreading",
            type: "string",
            render: (data, _, row, meta) => {
                if (0 !== row[meta.col - 1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(params.id, getCommonDtOptions(tableColumns));
};

/////////////////////////////////////////////////////////////////////
//
const getDataTable = (mapOfParams) => {
    // new rust code converts it to map so have to convert it back to
    // JSON Object (or TODO:re-write all this)
    const params = Object.fromEntries(mapOfParams);
    // console.log("Get Data Table 2: ", params);
    if (params.reportType === "quick") {
        return getQuickViewReportDataTable(params);
    } else if (params.reportType === "distributionPoints") {
        return getDistPointsViewReportDataTable(params);
    } else if (params.reportType === "deliveries") {
        return getDeliveriesViewReportDataTable(params);
    } else if (params.reportType === "full") {
        return getFullViewReportDataTable(params);
    } else if (params.reportType === "verification") {
        return getOrderVerificationViewReportDataTable(params);
    } else if (params.reportType === "spreadingJobs") {
        return getSpreadingJobsViewReportDataTable(params);
    } else if (params.reportType === "moneyCollection") {
        return getMoneyCollectionReportDataTable(params);
    } else if (params.reportType === "spreadingJobsUnfinished") {
        return getSpreadingJobsUnfinishedViewReportDataTable(params);
    }
    return undefined;
};

/////////////////////////////////////////////////////////////////////
//
const removeRowWithTr = (dt, tr) => {
    const row = dt.row(tr);
    row.remove().draw();
};

/////////////////////////////////////////////////////////////////////
//
const setSpreadersWithTr = (dt, tr, spreaders) => {
    const row = dt.row(tr);
    //const rowData = row.data();
    const spreadersIdx = dt.column('Spreaders:name').index();
    //rowData[spreadersIdx] = spreaders;
    dt.cell(row.index(), spreadersIdx).data(spreaders).draw();
    //row.data(rowData).draw();
};

export { getDataTable, removeRowWithTr, setSpreadersWithTr }
