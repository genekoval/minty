customElements.define(
    'minty-select',
    class extends HTMLElement {
        static formAssociated = true;

        constructor() {
            super();

            this.attachShadow({ mode: 'open' }).innerHTML =
                `<style>
                    :host {
                        position: relative;
                    }
                    button {
                        padding: .4rem;
                        font-size: 1rem;
                        height: 100%;
                        color: var(--text);
                        background: none;
                        border: none;
                        border-radius: calc(.5rem - 2px);

                        &:focus {
                            outline: none;
                        }
                    }
                    .closed:hover {
                        background: var(--secondary-background);
                    }
                    .menu {
                        background: hsl(var(--primary-background) / .6);
                        backdrop-filter: blur(8px);
                        position: absolute;
                        left: 50%;
                        transform: translate(-50%, .25rem);
                        border: 1px solid var(--divider);
                        border-radius: calc(.5rem - 2px);
                        padding: .25rem;
                    }
                    .menu > div {
                        padding: .5rem;
                        cursor: pointer;
                        border-radius: calc(.5rem - 2px);

                        &:hover {
                            background: var(--secondary-background);
                        }

                        &:focus {
                            background: var(--secondary-background);
                            outline: none;
                        }
                    }
                    .closed + .menu {
                        display: none;
                    }
                    .option {
                        display: flex;
                    }
                    .option minty-icon {
                        width: 1rem;
                        height: 1rem;
                    }
                    .option minty-title {
                        transform: translate(.4em, -.15em);
                    }
                </style>
                <button class="closed">
                    <slot></slot>
                </button>
                <div class="menu"></div>
                `;
            this.internals = this.attachInternals();
            this.menu = this.shadowRoot.querySelector('.menu');
            this.closeListener = event => this.handleCloseEvent(event);

            this.button = this.shadowRoot.querySelector('button');
            const slot = this.button.querySelector('slot');

            slot.addEventListener('slotchange', event => {
                const elements = event.target.assignedElements();
                const first = elements[0];

                if (first) {
                    const value = first.getAttribute('value');

                    this.value = value;
                    this.internals.setFormValue(value);

                    const icon = first.querySelector('minty-icon');
                    slot.outerHTML = icon.innerHTML;
                }

                const menu = this.shadowRoot.querySelector('.menu');

                for (let i = 0; i < elements.length; i++) {
                    const element = elements[i];
                    const value = element.getAttribute('value');
                    const div = document.createElement('div');

                    div.setAttribute('tabindex', '0');

                    div.classList.add('option');
                    div.innerHTML = element.innerHTML;

                    div.addEventListener('click', event => {
                        event.stopPropagation();
                        this.select(div, value);
                    });

                    div.addEventListener('keydown', event => {
                        if (event.key === 'Enter') {
                            event.stopPropagation();
                            this.select(div, value);
                        }
                    });

                    menu.appendChild(div);
                }
            });
        }

        connectedCallback() {
            this.button.addEventListener('click', event => {
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
);
