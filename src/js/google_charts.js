

const drawGoogleChart = (params) => {
        // Draw Charts
        const drawCharts=()=>{

            const options = {
                is3D: true,
                legend: 'left'
            };

            const patrolStandingsData = new google.visualization.DataTable();
            patrolStandingsData.addColumn('string', 'Patrol Sales');
            patrolStandingsData.addColumn('number', 'Amount Sold');

            for (const [group, amount] of params.groupRankings) {
                patrolStandingsData.addRow([group, amount]);
            }

            const patrolStandingsChart = new google.visualization.PieChart(
                document.getElementById('patrolStandingsChart'));
            patrolStandingsChart.draw(patrolStandingsData, options);


        };
        // Load the Visualization API and the corechart package.
        google.charts.load('current', {'packages':['corechart']});
        // Set a callback to run when the Google Visualization API is loaded.
        google.charts.setOnLoadCallback(drawCharts);
};

export {drawGoogleChart}
