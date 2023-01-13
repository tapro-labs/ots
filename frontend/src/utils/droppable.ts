export type OnDropFunction = (event: Event, files: File[]) => void;

export type DroppableOptions = {
  container: HTMLElement;
  onDrop?: OnDropFunction;
  multiple?: boolean;
  className?: string;
};

class Droppable {
  protected className: string;
  protected multiple: boolean;
  protected windowDragCounter = 0;
  protected containerDragCounter = 0;
  protected onDrop: OnDropFunction;
  protected container: DroppableOptions['container'];
  protected events: { [key: string]: Function[] } = {};

  constructor({ container, onDrop = () => {}, multiple = false, className = 'droppable' }: DroppableOptions) {
    if (!container) {
      throw new Error("'options.container' is requried");
    }

    this.onDrop = onDrop;
    this.container = container;
    this.className = className;
    this.multiple = multiple;

    this.handleDrop = this.handleDrop.bind(this);
    this.handleDragOver = this.handleDragOver.bind(this);
    this.handleDragEnter = this.handleDragEnter.bind(this);
    this.handleDragLeave = this.handleDragLeave.bind(this);
    this.handleWindowDragEnter = this.handleWindowDragEnter.bind(this);
    this.handleWindowDragLeave = this.handleWindowDragLeave.bind(this);

    this.boot();
  }

  protected get activeClassName() {
    return `${this.className}--active`;
  }

  protected get overClassName() {
    return `${this.className}--over`;
  }

  addEventListener(eventName: string, callback: (event: Event) => void) {
    if (!Object.prototype.hasOwnProperty.call(this.events, eventName)) {
      this.events[eventName] = [];
    }

    this.events[eventName].push(callback);
  }

  removeEventListener(eventName: string, callback: (event: Event) => void) {
    if (!Object.prototype.hasOwnProperty.call(this.events, eventName)) {
      return;
    }

    this.events[eventName] = this.events[eventName].filter(fnc => fnc !== callback);
  }

  boot() {
    this.bind();
    this.container.classList.add(this.className);
  }

  bind() {
    this.container.addEventListener('drop', this.handleDrop);
    this.container.addEventListener('dragover', this.handleDragOver);
    this.container.addEventListener('dragenter', this.handleDragEnter);
    this.container.addEventListener('dragleave', this.handleDragLeave);
    window.addEventListener('dragenter', this.handleWindowDragEnter);
    window.addEventListener('dragleave', this.handleWindowDragLeave);
  }

  unbind() {
    this.container.removeEventListener('drop', this.handleDrop);
    this.container.removeEventListener('dragover', this.handleDragOver);
    this.container.removeEventListener('dragenter', this.handleDragEnter);
    this.container.removeEventListener('dragleave', this.handleDragEnter);
    window.removeEventListener('dragenter', this.handleWindowDragEnter);
    window.removeEventListener('dragleave', this.handleWindowDragLeave);
  }

  destroy() {
    this.unbind();
    this.container.classList.remove(this.className);
    this.container.classList.remove(this.overClassName);
    this.container.classList.remove(this.activeClassName);
    this.events = {};
  }

  handleDrop(event: DragEvent) {
    event.preventDefault();

    this.windowDragCounter = 0;
    this.containerDragCounter = 0;

    this.container.classList.remove(this.overClassName);

    const droppables = document.querySelectorAll('.droppable--active');

    droppables.forEach(node => {
      node.classList.remove(this.activeClassName);
    });

    const realTarget = event.target as HTMLInputElement;
    const target = event.dataTransfer || realTarget;

    this.onDrop(event, this.getFiles(target));
  }

  handleDragOver(event: Event) {
    event.preventDefault();
    event.stopPropagation();
  }

  handleDragEnter(event: Event) {
    this.containerDragCounter++;

    this.container.classList.add(this.overClassName);

    if (Object.prototype.hasOwnProperty.call(this.events, 'dragEnter')) {
      this.events['dragEnter'].forEach(callback => callback(event));
    }
  }

  handleDragLeave(event: Event) {
    this.containerDragCounter--;

    if (this.containerDragCounter <= 0) {
      this.container.classList.remove(this.overClassName);
    }

    if (Object.prototype.hasOwnProperty.call(this.events, 'dragLeave')) {
      this.events['dragLeave'].forEach(callback => callback(event));
    }
  }

  handleWindowDragEnter(event: Event) {
    this.windowDragCounter++;

    this.container.classList.add(this.activeClassName);

    if (Object.prototype.hasOwnProperty.call(this.events, 'dragActive')) {
      this.events['dragActive'].forEach(callback => callback(event));
    }
  }

  handleWindowDragLeave(event: Event) {
    this.windowDragCounter--;

    if (this.windowDragCounter <= 0) {
      this.container.classList.remove(this.activeClassName);
    }

    if (Object.prototype.hasOwnProperty.call(this.events, 'dragActiveLeave')) {
      this.events['dragActiveLeave'].forEach(callback => callback(event));
    }
  }

  protected getFiles(target: HTMLInputElement | DataTransfer): File[] {
    if (!target?.files?.length) {
      return [];
    }

    if (this.multiple) {
      return Array.from(target.files);
    }

    return [target.files[0]];
  }
}

export default Droppable;
