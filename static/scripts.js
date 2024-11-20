let sections = document.querySelectorAll('section');

window.onload = function () {
    for (let i = 0; i < sections.length; i++) {
        sections[i].classList.add('hidden');
    }

    let current = 0;
    setInterval(function () {
        if (current === sections.length) {
            return;
        }
        activate(current);
        current++;
    }, 500);
}

let activate = function (index) {
    sections[index].classList.remove('hidden');
    sections[index].classList.add('active');
}