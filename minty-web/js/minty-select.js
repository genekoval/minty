import FormComponent from './form-component';

const NAME = 'minty-select';

export default class MintySelect extends FormComponent {
    constructor() {
        super(NAME);

        this.menu = this.shadowRoot.querySelector('.menu');
        this.button = this.shadowRoot.querySelector('button');
        this.closeListener = (event) => this.handleCloseEvent(event);

        const slot = this.button.querySelector('slot');

        slot.addEventListener('slotchange', (event) => {
            const elements = event.target.assignedElements();
            const first = elements[0];

            if (first) {
                const value = first.getAttribute('value');

                this.value = value;
                this.internals.setFormValue(value);

                const icon = first.querySelector('minty-icon');
                slot.outerHTML = icon.innerHTML;
            }

            for (let i = 0; i < elements.length; i++) {
                const element = elements[i];
                const value = element.getAttribute('value');
                const div = document.createElement('div');

                div.setAttribute('tabindex', '0');

                div.classList.add('option');
                div.innerHTML = element.innerHTML;

                div.addEventListener('click', (event) => {
                    event.stopPropagation();
                    this.select(div, value);
                });

                div.addEventListener('keydown', (event) => {
                    if (event.key === 'Enter') {
                        event.stopPropagation();
                        this.select(div, value);
                    }
                });

                this.menu.appendChild(div);
            }
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

    select(option, value) {
        const icon = option.querySelector('minty-icon');
        this.button.innerHTML = icon.innerHTML;

        this.internals.setFormValue(value);

        this.dispatchEvent(new Event('change', { bubbles: true }));
        this.toggleMenu();
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

customElements.define(NAME, MintySelect);
