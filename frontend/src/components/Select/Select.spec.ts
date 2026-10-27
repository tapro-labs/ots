import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { ModelSelect } from 'vue-search-select';
import TaproSelect from '@/components/Select/Select.vue';

const options = [
  { text: '5 minutes', value: 300 },
  { text: '1 hour', value: 3600 },
];

describe('Select', () => {
  it('passes the value and options down to the search select', () => {
    const wrapper = mount(TaproSelect, { props: { modelValue: 3600, options } });
    const select = wrapper.getComponent(ModelSelect);

    expect(select.props('modelValue')).toBe(3600);
    expect(select.props('options')).toEqual(options);
    expect(wrapper.text()).toContain('1 hour');
  });

  it('emits update:modelValue when an option is picked', async () => {
    const wrapper = mount(TaproSelect, { props: { modelValue: 3600, options } });

    await wrapper
      .findAll('.item')
      .find(item => item.text() === '5 minutes')
      ?.trigger('click');

    expect(wrapper.emitted('update:modelValue')).toEqual([[300]]);
  });

  it('reflects external modelValue changes', async () => {
    const wrapper = mount(TaproSelect, { props: { modelValue: 3600, options } });

    await wrapper.setProps({ modelValue: 300 });

    expect(wrapper.getComponent(ModelSelect).props('modelValue')).toBe(300);
  });
});
