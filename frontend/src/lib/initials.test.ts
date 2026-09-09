import { describe, expect, it } from 'vitest';
import { getInitials } from './initials';

describe('getInitials', () => {
  it('takes the first and last name initials, upper-cased', () => {
    expect(getInitials('Ada Lovelace')).toBe('AL');
    expect(getInitials('ada')).toBe('A');
    expect(getInitials('Some Long Person Name')).toBe('SN');
    expect(getInitials('  padded name ')).toBe('PN');
  });

  it('is empty without a name', () => {
    expect(getInitials()).toBe('');
    expect(getInitials('')).toBe('');
  });
});
