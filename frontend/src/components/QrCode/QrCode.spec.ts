import { beforeEach, describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import QrCodeBox from 'qrcode.vue';
import QrCode from '@/components/QrCode/QrCode.vue';

// jsdom has no canvas implementation, the QR rendering itself belongs to qrcode.vue
const global = { stubs: { QrCodeBox: true } };

describe('QrCode', () => {
  beforeEach(() => {
    // jsdom does not implement <dialog> methods
    HTMLDialogElement.prototype.showModal = vi.fn();
    HTMLDialogElement.prototype.close = vi.fn();
  });

  it('renders a QR code for the given text', () => {
    const wrapper = mount(QrCode, { global, props: { text: 'https://ots/secret/1#key' } });

    expect(wrapper.getComponent(QrCodeBox).props('value')).toBe('https://ots/secret/1#key');
  });

  it('opens the modal from the button and closes it via the close button or backdrop', async () => {
    const wrapper = mount(QrCode, { global, props: { text: 'x' } });
    const dialog = wrapper.get('dialog').element as HTMLDialogElement;

    await wrapper.get('button').trigger('click');
    expect(dialog.showModal).toHaveBeenCalledOnce();

    await wrapper.get('.modal-action button').trigger('click');
    await wrapper.get('dialog').trigger('click');
    expect(dialog.close).toHaveBeenCalledTimes(2);
  });

  it('keeps the modal open when clicking inside its content', async () => {
    const wrapper = mount(QrCode, { global, props: { text: 'x' } });

    await wrapper.get('.modal-box h3').trigger('click');

    expect((wrapper.get('dialog').element as HTMLDialogElement).close).not.toHaveBeenCalled();
  });

  it('applies extra attributes to the trigger button', () => {
    const wrapper = mount(QrCode, { global, props: { text: 'x' }, attrs: { 'data-test': 'qr' } });

    expect(wrapper.get('button').attributes('data-test')).toBe('qr');
    expect(wrapper.get('dialog').attributes('data-test')).toBeUndefined();
  });
});
