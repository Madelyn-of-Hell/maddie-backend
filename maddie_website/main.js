let root;
document.addEventListener(
    "DOMContentLoaded",
    (event) => {
        root = document.querySelector(":root")

        randomise_offsets()
    }
)
document.addEventListener(
    "mousemove",
    set_gradient_position
)

cookieStore.addEventListener(
    "change",
    (_) => {
        location.reload()
    }
)
function randomise_offsets() {
    for (let element of document.getElementsByClassName("left-box")) {
        let offset = (Math.random() + 0.5) * 10
        element.style.marginLeft = `${offset}%`
    }
    for (let element of document.getElementsByClassName("right-box")) {
        let offset = Math.random() * 10
        element.style.marginLeft = `${50 + offset}%`
    }
}


let target_x = 0;
let target_y = 0;
function set_gradient_position(event) {
    target_x = event.pageX;
    target_y = event.pageY;
}
function update_position() {
    let curr_x = Number(root.style.getPropertyValue('--mouse-x').slice(0,-2))
    let curr_y = Number(root.style.getPropertyValue('--mouse-y').slice(0,-2))
    root.style.setProperty('--mouse-x', `${curr_x + (target_x - curr_x)*0.05}px`)
    root.style.setProperty('--mouse-y', `${curr_y + (target_y - curr_y)*0.05}px`)
}
setInterval(
    update_position,
    10
)