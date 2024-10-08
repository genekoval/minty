export function clamp(num, min, max) {
    return Math.min(Math.max(num, min), max);
}

export function debounce(func, delay) {
    let timeout;

    return (...args) => {
        if (!timeout) {
            func.apply(this, args);

            timeout = setTimeout(() => {
                timeout = null;
            }, delay);
        }
    };
}
