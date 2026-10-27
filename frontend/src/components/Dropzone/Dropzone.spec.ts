import { afterEach, describe, expect, it, vi } from 'vitest';
import { mount, VueWrapper } from '@vue/test-utils';
import { h } from 'vue';
import Dropzone from '@/components/Dropzone/Dropzone.vue';
import { dragEvent, setInputFiles } from '@/test-utils/dom';

type SlotProps = {
  dragActive: boolean;
  dragOver: boolean;
  errorMessage: string;
  inputId: string;
  activateUploadFilePrompt: () => void;
};

const wrappers: VueWrapper[] = [];

const mountDropzone = (props: Record<string, unknown> = {}) => {
  const wrapper = mount(Dropzone, {
    props,
    attachTo: document.body,
    slots: {
      default: (slotProps: SlotProps) =>
        h(
          'div',
          {
            class: 'zone',
            'data-over': String(slotProps.dragOver),
            'data-active': String(slotProps.dragActive),
            'data-input-id': slotProps.inputId,
            onClick: slotProps.activateUploadFilePrompt,
          },
          slotProps.errorMessage
        ),
    },
  });

  wrappers.push(wrapper);

  return wrapper;
};

describe('Dropzone', () => {
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount());
  });

  it('refuses to render without slot content', () => {
    vi.spyOn(console, 'warn').mockImplementation(() => {});

    expect(() => mount(Dropzone)).toThrow(/renderless component/);
  });

  it('exposes the id of its hidden file input and the accepted types', () => {
    const wrapper = mountDropzone({ inputAccepted: 'image/*' });
    const input = wrapper.get('input[type="file"]');

    expect(wrapper.get('.zone').attributes('data-input-id')).toBe(input.attributes('id'));
    expect(input.attributes('accept')).toBe('image/*');
  });

  it('emits success with the file dropped onto the wrapped element', async () => {
    const file = new File(['hello'], 'hello.txt');
    const wrapper = mountDropzone();

    wrapper.get('.zone').element.dispatchEvent(dragEvent('drop', [file]));

    expect(wrapper.emitted('success')).toEqual([[file]]);
    expect(wrapper.emitted('error')).toBeUndefined();
  });

  it('emits success with the file picked through the input', async () => {
    const file = new File(['hello'], 'picked.txt');
    const wrapper = mountDropzone();
    const input = wrapper.get('input[type="file"]');

    setInputFiles(input.element as HTMLInputElement, [file]);
    await input.trigger('change');

    expect(wrapper.emitted('success')).toEqual([[file]]);
  });

  it('emits an error and exposes its message when no file was provided', async () => {
    const wrapper = mountDropzone();

    wrapper.get('.zone').element.dispatchEvent(dragEvent('drop', []));
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('error')?.[0][0]).toEqual(new Error('No file was provided!'));
    expect(wrapper.get('.zone').text()).toBe('No file was provided!');
  });

  it('emits the validation error instead of success and clears it on the next valid file', async () => {
    const validation = vi.fn((file: File) => (file.name.endsWith('.exe') ? new Error('No executables') : undefined));
    const wrapper = mountDropzone({ validation });
    const zone = wrapper.get('.zone');

    zone.element.dispatchEvent(dragEvent('drop', [new File([''], 'virus.exe')]));
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('error')?.[0][0]).toEqual(new Error('No executables'));
    expect(wrapper.emitted('success')).toBeUndefined();
    expect(zone.text()).toBe('No executables');

    const valid = new File([''], 'notes.txt');

    zone.element.dispatchEvent(dragEvent('drop', [valid]));
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('success')).toEqual([[valid]]);
    expect(zone.text()).toBe('');
  });

  it('reflects drag state through slot props', async () => {
    const wrapper = mountDropzone();
    const zone = wrapper.get('.zone');

    window.dispatchEvent(dragEvent('dragenter'));
    zone.element.dispatchEvent(dragEvent('dragenter'));
    await wrapper.vm.$nextTick();
    expect(zone.attributes('data-active')).toBe('true');
    expect(zone.attributes('data-over')).toBe('true');

    zone.element.dispatchEvent(dragEvent('dragleave'));
    window.dispatchEvent(dragEvent('dragleave'));
    await wrapper.vm.$nextTick();
    expect(zone.attributes('data-active')).toBe('false');
    expect(zone.attributes('data-over')).toBe('false');
  });

  it('opens the file picker through activateUploadFilePrompt', async () => {
    const wrapper = mountDropzone();
    const click = vi.spyOn(wrapper.get('input[type="file"]').element as HTMLInputElement, 'click');

    await wrapper.get('.zone').trigger('click');

    expect(click).toHaveBeenCalledOnce();
  });

  it('stops listening for drops after unmount', () => {
    const wrapper = mountDropzone();
    const zone = wrapper.get('.zone').element;

    wrappers.splice(0);
    wrapper.unmount();
    zone.dispatchEvent(dragEvent('drop', [new File([''], 'late.txt')]));

    expect(wrapper.emitted('success')).toBeUndefined();
  });
});
