function updateCalendar() {
    fetch("/api/data")
        .then(res => res.json())
        .then(events => {
            var calendarEl = document.getElementById("calendar");
            var calendar = new FullCalendar.Calendar(calendarEl, {
                initialView: "dayGridMonth",
                headerToolbar: {
                    "left": "prev,next today",
                    "center": "title",
                    "right": "dayGridMonth,timeGridWeek,timeGridDay,listWeek"
                },
                events: events
            });
            calendar.render();
        });
}
