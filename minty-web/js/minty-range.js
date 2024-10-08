import { clamp, debounce } from './functions';
import WebComponent from './web-component';

const NAME = 'minty-range';

export default class MintyRange extends WebComponent {
    static observedAttributes = ['min', 'max', 'value'];

    get seeking() {
        return this.track.classList.contains('seeking');
    }

    set seeking(val) {
        if (val == true) {
            this.track.classList.add('seeking');
            this.dispatchEvent(new Event('seeking'));
        } else if (val == false) {
            this.track.classList.remove('seeking');
            this.dispatchEvent(new Event('seeked'));
        }
    }

    get value() {
        return this._value ?? this.getAttribute('min');
    }

    set value(val) {
        this._value = val;
        this.updateDebounced();
    }

    constructor() {
        super(NAME);

        this.track = this.shadowRoot.getElementById('track');
        this.fill = this.shadowRoot.getElementById('fill');
        this.thumb = this.shadowRoot.getElementById('thumb');
        this.updateDebounced = debounce(() => this.update(), 1000);
    }

    attributeChangedCallback(name, _oldValue, newValue) {
        if (name == 'value') {
            this.value = newValue;
        } else {
            this.update();
        }
    }

    connectedCallback() {
        this.addEventListener('mousedown', (e) => this.handleMousedown(e));
    }

    /**
     * @param {MouseEvent} event The mousedown event.
     */
    handleMousedown(event) {
        if (event.buttons != 1) return;

        event.preventDefault();

        this.seeking = true;
        this.setPosition(event.pageX);

        const onMousemove = (e) => {
            this.setPosition(e.pageX);
            this.dispatchEvent(new Event('input', { bubbles: true }));
        };

        document.addEventListener('mousemove', onMousemove);

        const onMouseup = (e) => {
            if (e.button != 0) return;

            this.seeking = false;

            document.removeEventListener('mousemove', onMousemove);
            document.removeEventListener('mouseup', onMouseup);

            this.dispatchEvent(new Event('change', { bubbles: true }));
        };

        document.addEventListener('mouseup', onMouseup);
    }

    /**
     * Sets the range's value based on pointer position.
     *
     * @param {double} pageX The X coordinate of the mouse pointer relative to the whole document.
     */
    setPosition(pageX) {
        const min = this.getAttribute('min');
        const max = this.getAttribute('max');
        const rect = this.getBoundingClientRect();
        const pos = (pageX - rect.left) / this.offsetWidth;
        const value = clamp(pos * max, min, max);

        this._value = value;
        this.update();
    }

    /**
     * @param {double} percent The amount of the range to fill.
     */
    setPercentage(percent) {
        const value = `${percent}%`;

        this.fill.style.width = value;
        this.thumb.style.left = value;
    }

    update() {
        const min = this.getAttribute('min') ?? 0;
        const max = this.getAttribute('max') ?? 1;
        const diff = max - min;

        if (diff == 0) return;

        this.setPercentage((this.value / diff) * 100);
    }
}

customElements.define(NAME, MintyRange);
