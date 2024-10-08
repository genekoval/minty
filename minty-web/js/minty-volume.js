import WebComponent from './web-component';

const NAME = 'minty-volume';

export default class MintyVolume extends WebComponent {
    get isMuted() {
        return this.state === 'mute';
    }

    get state() {
        return this.button.getAttribute('data-state');
    }

    set state(value) {
        this.button.setAttribute('data-state', value);
    }

    get value() {
        return this.range.value;
    }

    constructor() {
        super(NAME);

        this.button = this.shadowRoot.querySelector('button');
        this.range = this.shadowRoot.querySelector('minty-range');
    }

    connectedCallback() {
        this.button.addEventListener('click', () => this.toggleMute());
        this.range.addEventListener('input', () => this.setState());
    }

    mute() {
        this.previousValue = this.range.value;
        this.range.value = 0;

        this.setState();
    }

    restoreVolume() {
        this.range.value = this.previousValue;

        this.setState();
    }

    setState() {
        const value = this.range.value;

        if (value == 0) this.state = 'mute';
        else if (value < 0.5) this.state = 'low';
        else this.state = 'high';

        this.dispatchEvent(new Event('change', { bubbles: true }));
    }

    toggleMute() {
        if (this.isMuted) this.restoreVolume();
        else this.mute();
    }
}

customElements.define(NAME, MintyVolume);
