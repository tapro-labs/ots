import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import copy from 'copy-to-clipboard';
import CopyToClipboard from '@/components/CopyToClipboard/CopyToClipboard.vue';
import useNotifications from '@/composables/useNotifications';

// copy-to-clipboard v4 is async and resolves to whether the copy worked
vi.mock('copy-to-clipboard', () => ({ default: vi.fn(async () => true) }));

describe('CopyToClipboard', () => {
  const cleanups: (() => void)[] = [];

  const listenToNotifications = () => {
    const listener = vi.fn();
    const host = mount({
      setup() {
        useNotifications().onMessageReceived(listener);

        return () => null;
      },
    });

    cleanups.push(() => host.unmount());

    return listener;
  };

  afterEach(() => {
    cleanups.splice(0).forEach(cleanup => cleanup());
    vi.mocked(copy).mockClear();
  });

  it('copies the current text when clicked', async () => {
    const wrapper = mount(CopyToClipboard, { props: { text: 'https://ots/secret/1#key' } });

    await wrapper.get('button').trigger('click');
    expect(copy).toHaveBeenCalledWith('https://ots/secret/1#key');

    await wrapper.setProps({ text: 'updated' });
    await wrapper.get('button').trigger('click');
    expect(copy).toHaveBeenLastCalledWith('updated');
  });

  it('shows a success notification when a message is provided', async () => {
    const listener = listenToNotifications();
    const wrapper = mount(CopyToClipboard, { props: { text: 'abc', successMessage: 'Copied!' } });

    await wrapper.get('button').trigger('click');
    await flushPromises();

    expect(listener).toHaveBeenCalledOnce();
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({ message: 'Copied!', error: false }));
  });

  it('does not notify when no success message is provided', async () => {
    const listener = listenToNotifications();
    const wrapper = mount(CopyToClipboard, { props: { text: 'abc' } });

    await wrapper.get('button').trigger('click');
    await flushPromises();

    expect(copy).toHaveBeenCalledOnce();
    expect(listener).not.toHaveBeenCalled();
  });

  it('waits for the copy to finish before showing the success notification', async () => {
    let finishCopy!: (copied: boolean) => void;
    vi.mocked(copy).mockImplementationOnce(() => new Promise<boolean>(resolve => (finishCopy = resolve)));
    const listener = listenToNotifications();
    const wrapper = mount(CopyToClipboard, { props: { text: 'abc', successMessage: 'Copied!' } });

    await wrapper.get('button').trigger('click');
    await flushPromises();
    expect(listener).not.toHaveBeenCalled();

    finishCopy(true);
    await flushPromises();
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({ message: 'Copied!', error: false }));
  });

  it('shows an error notification instead of the success message when copying fails', async () => {
    vi.mocked(copy).mockResolvedValueOnce(false);
    const listener = listenToNotifications();
    const wrapper = mount(CopyToClipboard, { props: { text: 'abc', successMessage: 'Copied!' } });

    await wrapper.get('button').trigger('click');
    await flushPromises();

    expect(listener).toHaveBeenCalledOnce();
    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({ message: 'Could not copy to clipboard.', error: true })
    );
  });
});
