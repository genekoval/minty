import { debounce } from './functions';
import WebComponent from './web-component';

const NAME = 'minty-audio';

export default class MintyAudio extends WebComponent {
    static observedAttributes = ['autoplay', 'src'];

    constructor() {
        super(NAME);

        this.updateProgress = debounce(
            (value) => (this.range.value = value),
            1000,
        );
        this.seeking = false;
        this.audio = this.shadowRoot.querySelector('audio');
        this.range = this.shadowRoot.getElementById('range');
        this.primaryControls =
            this.shadowRoot.getElementById('primary-controls');
        this.playpause = this.shadowRoot.getElementById('playpause');
        this.time = this.shadowRoot.getElementById('time');
        this.duration = this.shadowRoot.getElementById('duration');
        this.volume = this.shadowRoot.querySelector('minty-volume');
        this.close = this.shadowRoot.getElementById('close');
    }

    connectedCallback() {
        this.audio.addEventListener('play', () => {
            this.primaryControls.setAttribute('data-state', 'play');
        });

        this.audio.addEventListener('pause', () => {
            this.primaryControls.setAttribute('data-state', 'pause');
        });

        this.audio.addEventListener('ended', () => {
            this.range.setPercentage(100);
        });

        this.audio.addEventListener('loadedmetadata', () => {
            this.setDuration();
        });

        this.audio.addEventListener('timeupdate', () => {
            this.setDuration();
            if (!this.seeking) this.setTime(this.audio.currentTime);
        });

        this.playpause.addEventListener('click', () => {
            if (this.audio.paused || this.audio.ended) {
                this.audio.play();
            } else {
                this.audio.pause();
            }
        });

        this.range.addEventListener('input', () => {
            if (this.seeking) this.setTime(this.range.value);
        });

        this.range.addEventListener('change', () => {
            this.audio.currentTime = this.range.value;
        });

        this.range.addEventListener('seeking', () => (this.seeking = true));
        this.range.addEventListener('seeked', () => (this.seeking = false));

        this.volume.addEventListener('change', () => {
            this.audio.volume = this.volume.value;
        });

        this.close.addEventListener('click', () => {
            this.dispatchEvent(new Event('close', { bubbles: true }));
        });
    }

    attributeChangedCallback(name, _oldValue, newValue) {
        this.audio.setAttribute(name, newValue);
    }

    setDuration() {
        if (this.range.getAttribute('max')) {
            return;
        }

        let seconds = this.audio.duration;

        this.range.setAttribute('max', seconds);

        this.hours = Math.floor(seconds / 3600);
        seconds %= 3600;

        this.minutes = Math.floor(seconds / 60);

        seconds = Math.floor(seconds % 60);

        if (this.hours > 9 || this.minutes > 9) {
            this.hours = this.hours.toString().padStart(2, '0');
            this.minutes = this.minutes.toString().padStart(2, '0');
        }

        seconds = seconds.toString().padStart(2, '0');

        let duration =
            this.hours > 0
                ? `${this.hours}:${this.minutes}:${seconds}`
                : `${this.minutes}:${seconds}`;

        this.duration.innerText = duration;
    }

    setTime(seconds) {
        if (!this.seeking) this.updateProgress(seconds);

        let hours = Math.floor(seconds / 3600);
        seconds %= 3600;

        let minutes = Math.floor(seconds / 60);

        seconds = Math.floor(seconds % 60);

        if (this.hours > 9 || this.minutes > 9) {
            hours = hours.toString().padStart(2, '0');
            minutes = minutes.toString().padStart(2, '0');
        }

        seconds = seconds.toString().padStart(2, '0');

        let time =
            this.hours > 0
                ? `${hours}:${minutes}:${seconds}`
                : `${minutes}:${seconds}`;

        this.time.innerText = time;
    }
}

customElements.define(NAME, MintyAudio);
