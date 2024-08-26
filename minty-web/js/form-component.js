import WebComponent from './web-component';

export default class FormComponent extends WebComponent {
    static formAssociated = true;

    constructor(name) {
        super(name);

        this.internals = this.attachInternals();
    }
}
