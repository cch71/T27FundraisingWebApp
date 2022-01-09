
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
                "csv", "copy", "excel", "pdf", "print", 'colvis'
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
        { title: "Bags" },
        { title: "Special Instructions" },
        { title: "Verified" },
        { title: "Money Collected" },
        { title: "Donations" },
        { title: "Cash" },
        { title: "Check" },
        { title: "Check Numbers" },
        { title: "Total Amount" },
        { title: "Order Owner", name: "OrderOwner", visible: params.showOrderOwner },
        { title: "Actions", "orderable": false, className: "all" }
    ];

    return new DataTable(
        params.id,
        {
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "pdf", "print", 'colvis'
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
                "csv", "copy", "excel", "pdf", "print", 'colvis'
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
    } else if (params.reportType === "full") {
        return getFullViewReportDataTable(params);
    } else if (params.reportType === "spreadingJobs") {
        return getSpreadingJobsViewReportDataTable(params);
    }
    return undefined;
};

/////////////////////////////////////////////////////////////////////
//
const removeRowWithTr = (dt, tr) => {
    const row = dt.row(tr);
    row.remove().draw();
};

export {getDataTable, removeRowWithTr}
