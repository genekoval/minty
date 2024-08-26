export default class WebComponent extends HTMLElement {
    constructor(name) {
        super();

        const content = document
            .getElementById(`${name}-template`)
            .content.cloneNode(true);

        this.attachShadow({ mode: 'open' }).appendChild(content);
    }
}
