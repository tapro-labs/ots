import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Droppable from '@/utils/droppable';
import { dragEvent } from '@/test-utils/dom';

describe('Droppable', () => {
  let container: HTMLElement;
  const instances: Droppable[] = [];

  const create = (options: Partial<ConstructorParameters<typeof Droppable>[0]> = {}) => {
    const droppable = new Droppable({ container, ...options });

    instances.push(droppable);

    return droppable;
  };

  beforeEach(() => {
    container = document.createElement('div');
    document.body.appendChild(container);
  });

  afterEach(() => {
    instances.splice(0).forEach(instance => instance.destroy());
    container.remove();
  });

  it('requires a container', () => {
    expect(() => new Droppable({ container: null as unknown as HTMLElement })).toThrow(/container/);
  });

  it('marks the container with the (custom) class name', () => {
    create({ className: 'zone' });

    expect(container.classList.contains('zone')).toBe(true);
  });

  it('passes only the first dropped file unless multiple is enabled', () => {
    const files = [new File(['a'], 'a.txt'), new File(['b'], 'b.txt')];
    const onDrop = vi.fn();

    create({ onDrop });
    container.dispatchEvent(dragEvent('drop', files));

    expect(onDrop).toHaveBeenCalledWith(expect.any(Event), [files[0]]);

    instances.splice(0).forEach(instance => instance.destroy());
    const onMultipleDrop = vi.fn();

    create({ onDrop: onMultipleDrop, multiple: true });
    container.dispatchEvent(dragEvent('drop', files));

    expect(onMultipleDrop).toHaveBeenCalledWith(expect.any(Event), files);
  });

  it('passes an empty list when nothing was dropped and prevents the browser default', () => {
    const onDrop = vi.fn();
    const event = dragEvent('drop', []);

    create({ onDrop });
    container.dispatchEvent(event);

    expect(onDrop).toHaveBeenCalledWith(event, []);
    expect(event.defaultPrevented).toBe(true);
  });

  it('prevents the default on dragover so dropping is allowed', () => {
    const event = dragEvent('dragover');

    create();
    container.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
  });

  it('tracks nested dragenter/dragleave on the container and only clears "over" on the last leave', () => {
    const onEnter = vi.fn();
    const onLeave = vi.fn();
    const droppable = create();

    droppable.addEventListener('dragEnter', onEnter);
    droppable.addEventListener('dragLeave', onLeave);

    container.dispatchEvent(dragEvent('dragenter'));
    container.dispatchEvent(dragEvent('dragenter')); // entering a child element
    expect(container.classList.contains('droppable--over')).toBe(true);

    container.dispatchEvent(dragEvent('dragleave'));
    expect(container.classList.contains('droppable--over')).toBe(true);

    container.dispatchEvent(dragEvent('dragleave'));
    expect(container.classList.contains('droppable--over')).toBe(false);
    expect(onEnter).toHaveBeenCalledTimes(2);
    expect(onLeave).toHaveBeenCalledTimes(2);
  });

  it('marks the container active while something is dragged over the window', () => {
    const onActive = vi.fn();
    const onActiveLeave = vi.fn();
    const droppable = create();

    droppable.addEventListener('dragActive', onActive);
    droppable.addEventListener('dragActiveLeave', onActiveLeave);

    window.dispatchEvent(dragEvent('dragenter'));
    expect(container.classList.contains('droppable--active')).toBe(true);

    window.dispatchEvent(dragEvent('dragleave'));
    expect(container.classList.contains('droppable--active')).toBe(false);
    expect(onActive).toHaveBeenCalledOnce();
    expect(onActiveLeave).toHaveBeenCalledOnce();
  });

  it('clears the active and over state after a drop', () => {
    create();

    window.dispatchEvent(dragEvent('dragenter'));
    container.dispatchEvent(dragEvent('dragenter'));
    container.dispatchEvent(dragEvent('drop', []));

    expect(container.classList.contains('droppable--over')).toBe(false);
    expect(container.classList.contains('droppable--active')).toBe(false);
  });

  it('clears the active state after a drop when a custom class name is used', () => {
    create({ className: 'zone' });

    window.dispatchEvent(dragEvent('dragenter'));
    expect(container.classList.contains('zone--active')).toBe(true);

    container.dispatchEvent(dragEvent('drop', []));
    expect(container.classList.contains('zone--active')).toBe(false);
  });

  it('removes listeners after they are unsubscribed', () => {
    const onEnter = vi.fn();
    const droppable = create();

    droppable.addEventListener('dragEnter', onEnter);
    droppable.removeEventListener('dragEnter', onEnter);
    droppable.removeEventListener('unknown', onEnter);
    container.dispatchEvent(dragEvent('dragenter'));

    expect(onEnter).not.toHaveBeenCalled();
  });

  it('stops reacting to every drag event after destroy', () => {
    const onDrop = vi.fn();
    const droppable = create({ onDrop });

    droppable.destroy();
    expect(container.classList.contains('droppable')).toBe(false);

    // listeners registered after destroy must not be reachable through stale DOM handlers
    const listener = vi.fn();

    ['dragEnter', 'dragLeave', 'dragActive', 'dragActiveLeave'].forEach(name =>
      droppable.addEventListener(name, listener)
    );
    ['dragenter', 'dragleave', 'drop'].forEach(type => container.dispatchEvent(dragEvent(type, [])));
    ['dragenter', 'dragleave'].forEach(type => window.dispatchEvent(dragEvent(type)));

    expect(listener).not.toHaveBeenCalled();
    expect(onDrop).not.toHaveBeenCalled();
    expect(container.className).toBe('');
  });
});
