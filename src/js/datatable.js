
/////////////////////////////////////////////////////////////////////
//
const getQuickViewReportDataTable = (params) => {
    const tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Delivery Date", name: "DeliveryDate" },
        { title: "Spreaders", name: "Spreaders", visible: false},
        {
            title: "Spreading",
            render: (data, _, row, meta) => {
                if (0!==row[meta.col-1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////
//
const getDeliveriesViewReportDataTable = (params) => {
    console.log("Setting Deliveries Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Delivery Date", name: "DeliveryDate", className: "all" },
        { title: "Name", className: "all" },
        { title: "Neighborhood" },
        { title: "Address" },
        { title: "Bags" },
        { title: "Phone" },
        { title: "Location" },
        { title: "Notes" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            order: [[ 1, "asc" ]],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////
//
const getDistPointsViewReportDataTable = (params) => {
    console.log("Setting Distrtibution Points Report View");
    let tableColumns = [
        { title: "Delivery Date", name: "DeliveryDate", className: "all" },
        { title: "Total Bags", name: "TotalBags", className: "all"},
    ];

    for (const header of params.distPoints) {
        tableColumns.push({title: header, className: "all"});
    }

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};



/////////////////////////////////////////////////////////////////////
//
const getFullViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Phone" },
        { title: "Email" },
        { title: "Address 1" },
        { title: "Address 2" },
        { title: "Neighborhood" },
        { title: "Delivery Date" },
        { title: "Spreaders", name: "Spreaders", visible: false },
        {
            title: "Spreading",
            render: (data, _, row, meta) => {
                if (0!==row[meta.col-1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Bags" },
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

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////////
//
const getOrderVerificationViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Delivery Date" },
        { title: "Donations" },
        { title: "Cash" },
        { title: "Check" },
        { title: "Check Numbers" },
        { title: "Total Amount" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Verified" },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////////
//
const getSpreadingJobsUnfinishedViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { title: "Order Owner", name: "OrderOwner" },
        { title: "Name" },
        { title: "Bags Left To Spread", className: "all" }
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////////
//
const getSpreadingJobsViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Phone", className: "all" },
        { title: "Delivery Date" },
        { title: "Instructions" },
        { title: "Address" },
        { title: "Neighborhood", className: "all"},
        { title: "Spreaders", name: "Spreaders", visible: false},
        {
            title: "Spreading",
            render: (data, _, row, meta) => {
                if (0!==row[meta.col-1].length) {
                    return `${data}: Spread`
                } else {
                    return data;
                }
            }
        },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};

/////////////////////////////////////////////////////////////////////
//
const getDataTable = (params) => {
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

export {getDataTable, removeRowWithTr, setSpreadersWithTr}
