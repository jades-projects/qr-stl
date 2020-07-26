// https://stackoverflow.com/a/62176999/2350164
const downloadURL = (data, fileName) => {
    const a = document.createElement('a')
    a.href = data
    a.download = fileName
    document.body.appendChild(a)
    a.style.display = 'none'
    a.click()
    a.remove()
}

const downloadBlob = (data, fileName, mimeType) => {
    const blob = new Blob([data], {
        type: mimeType
    })

    const url = window.URL.createObjectURL(blob)
    downloadURL(url, fileName)
    setTimeout(() => window.URL.revokeObjectURL(url), 1000)
}

import('../pkg/index.js')
    .catch(console.error)
    .then(mod => {
        const form = document.getElementById('generate_form');
        form.onsubmit = function (ev) {
            debugger;
            try {
                const stl = mod.qr2stl(
                    ev.target.elements.string_in.value,
                    ev.target.elements.base_height.value,
                    ev.target.elements.base_size.value,
                    ev.target.elements.pixel_size.value
                );
                downloadBlob(stl, 'qr.stl', 'application/octet-stream');
            } catch (e) {
                alert('Error making STL: ' + e);
            }
            return false;
        };
        const btn = document.getElementById('generate_button');
        btn.disabled = false;
    });