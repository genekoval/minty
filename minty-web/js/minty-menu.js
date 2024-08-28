import WebComponent from './web-component';

import htmx from 'htmx.org';

const NAME = 'minty-menu';

export default class MintyMenu extends WebComponent {
    constructor() {
        super(NAME);

        this.menu = this.shadowRoot.querySelector('.menu');
        this.button = this.shadowRoot.querySelector('button');
        this.closeListener = (event) => this.handleCloseEvent(event);

        const slot = this.menu.querySelector('slot');

        slot.addEventListener('slotchange', (event) => {
            const elements = event.target.assignedElements();

            if (elements.length === 0) {
                return;
            }

            this.menu.removeChild(slot);

            for (const element of elements) {
                this.menu.appendChild(element);
            }

            htmx.process(this.shadowRoot);
        });
    }

    connectedCallback() {
        this.button.addEventListener('click', (event) => {
            event.stopPropagation();
            this.toggleMenu();
        });
    }

    handleCloseEvent(event) {
        if (
            event.type === 'click' ||
            (event.type === 'keydown' && event.key === 'Escape')
        ) {
            event.stopPropagation();
            this.toggleMenu();
        }
    }

    toggleMenu() {
        if (this.button.classList.toggle('closed')) {
            document.removeEventListener('click', this.closeListener);
            document.removeEventListener('keydown', this.closeListener);
        } else {
            document.addEventListener('click', this.closeListener);
            document.addEventListener('keydown', this.closeListener);

            this.menu.focus();
        }
    }
}

customElements.define(NAME, MintyMenu);
