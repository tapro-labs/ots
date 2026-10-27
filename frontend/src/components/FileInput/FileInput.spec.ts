import { afterEach, describe, expect, it } from 'vitest';
import { mount, VueWrapper } from '@vue/test-utils';
import FileInput from '@/components/FileInput/FileInput.vue';
import FileTooLargeError from '@/exceptions/FileTooLargeError';
import { dragEvent, fileOfSize } from '@/test-utils/dom';

const KB = 1024;
const MB = 1024 * KB;
const GB = 1024 * MB;

const wrappers: VueWrapper[] = [];

const mountFileInput = (props: Record<string, unknown> = {}) => {
  const wrapper = mount(FileInput, {
    props,
    attrs: { class: 'my-input' },
    slots: { default: '<textarea />', icons: '<span class="extra-icon" />' },
    attachTo: document.body,
  });

  wrappers.push(wrapper);

  return wrapper;
};

const drop = (wrapper: VueWrapper, file: File) => {
  wrapper.get('.my-input').element.dispatchEvent(dragEvent('drop', [file]));
};

describe('FileInput', () => {
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount());
  });

  it('renders slots and forwards attributes to the drop area', () => {
    const wrapper = mountFileInput();

    expect(wrapper.find('.my-input textarea').exists()).toBe(true);
    expect(wrapper.find('.my-input .extra-icon').exists()).toBe(true);
  });

  it('links the upload label to the hidden file input', () => {
    const wrapper = mountFileInput();

    expect(wrapper.get('label').attributes('for')).toBe(wrapper.get('input[type="file"]').attributes('id'));
  });

  it.each([
    ['1MB', 1 * MB],
    ['40MB', 40 * MB],
    ['512KB', 512 * KB],
    ['2GB', 2 * GB],
  ])('accepts files up to %s and rejects anything larger', (maxSize, limit) => {
    const wrapper = mountFileInput({ maxSize });
    const atLimit = fileOfSize(limit);
    const overLimit = fileOfSize(limit + 1);

    drop(wrapper, atLimit);
    drop(wrapper, overLimit);

    expect(wrapper.emitted('success')).toEqual([[atLimit]]);

    const [[error]] = wrapper.emitted('error') as [[FileTooLargeError]];

    expect(error).toBeInstanceOf(FileTooLargeError);
    expect(error.size).toBe(limit + 1);
  });

  it('defaults to a 1MB limit', () => {
    const wrapper = mountFileInput();

    drop(wrapper, fileOfSize(MB + 1));

    expect(wrapper.emitted('error')?.[0][0]).toBeInstanceOf(FileTooLargeError);
  });

  it.each([
    ['40mb', 40 * MB],
    ['500B', 500],
    ['2048', 2048],
    ['1.5MB', 1.5 * MB],
  ])('understands the size notation %s', (maxSize, limit) => {
    const wrapper = mountFileInput({ maxSize });

    drop(wrapper, fileOfSize(limit));
    drop(wrapper, fileOfSize(limit + 1));

    expect(wrapper.emitted('success')).toHaveLength(1);
    expect(wrapper.emitted('error')).toHaveLength(1);
  });

  it('forwards the "no file" error from the dropzone', () => {
    const wrapper = mountFileInput();

    wrapper.get('.my-input').element.dispatchEvent(dragEvent('drop', []));

    expect(wrapper.emitted('error')?.[0][0]).toEqual(new Error('No file was provided!'));
  });
});
