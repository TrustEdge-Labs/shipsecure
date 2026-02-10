#!/usr/bin/env node

/**
 * WCAG 2.1 Contrast Ratio Calculator for OKLch colors
 *
 * Converts OKLch to sRGB, then calculates relative luminance per WCAG spec.
 * Minimum requirements:
 * - Normal text: 4.5:1 (AA)
 * - Large text: 3:1 (AA)
 */

// OKLch to sRGB conversion (via Lab and XYZ)
function oklchToRgb(L, C, H) {
  // Convert to Lab
  const hRad = (H * Math.PI) / 180;
  const a = C * Math.cos(hRad);
  const b = C * Math.sin(hRad);

  // OKLab uses a different transform than CIE Lab
  // Simplified approximation for perceptual lightness
  const l = L;

  // Convert OKLab to linear RGB (D65 white point)
  const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
  const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
  const s_ = l - 0.0894841775 * a - 1.2914855480 * b;

  const l3 = l_ * l_ * l_;
  const m3 = m_ * m_ * m_;
  const s3 = s_ * s_ * s_;

  let r = +4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
  let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
  let b_ = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

  // Clamp and apply sRGB gamma
  r = sRGBGamma(Math.max(0, Math.min(1, r)));
  g = sRGBGamma(Math.max(0, Math.min(1, g)));
  b_ = sRGBGamma(Math.max(0, Math.min(1, b_)));

  return [r, g, b_];
}

function sRGBGamma(v) {
  if (v <= 0.0031308) {
    return 12.92 * v;
  }
  return 1.055 * Math.pow(v, 1 / 2.4) - 0.055;
}

function relativeLuminance(r, g, b) {
  // Inverse sRGB gamma to get linear RGB
  const rsRGB = r <= 0.04045 ? r / 12.92 : Math.pow((r + 0.055) / 1.055, 2.4);
  const gsRGB = g <= 0.04045 ? g / 12.92 : Math.pow((g + 0.055) / 1.055, 2.4);
  const bsRGB = b <= 0.04045 ? b / 12.92 : Math.pow((b + 0.055) / 1.055, 2.4);

  return 0.2126 * rsRGB + 0.7152 * gsRGB + 0.0722 * bsRGB;
}

function contrastRatio(L1, L2) {
  const lighter = Math.max(L1, L2);
  const darker = Math.min(L1, L2);
  return (lighter + 0.05) / (darker + 0.05);
}

// Test pairs from the plan
const colors = {
  // Light mode primitives
  white: [1.000, 0, 0],
  'gray-50': [0.985, 0.002, 247],
  'gray-100': [0.967, 0.003, 264],
  'gray-200': [0.928, 0.006, 264],
  'gray-300': [0.872, 0.010, 258],
  'gray-400': [0.650, 0.015, 261],
  'gray-500': [0.530, 0.018, 264],
  'gray-600': [0.446, 0.014, 264],
  'gray-700': [0.373, 0.011, 264],
  'gray-800': [0.278, 0.009, 264],
  'gray-900': [0.210, 0.006, 264],
  'gray-950': [0.130, 0.004, 264],
  'blue-600': [0.546, 0.245, 262],
  'blue-700': [0.488, 0.243, 264],
  'blue-800': [0.424, 0.199, 265],
  'green-50': [0.981, 0.020, 155],
  'green-800': [0.473, 0.112, 152],
  'green-950': [0.240, 0.065, 155],
  'red-50': [0.971, 0.013, 17],
  'red-700': [0.505, 0.175, 27],
  'red-800': [0.440, 0.145, 27],
  'red-950': [0.258, 0.078, 28],
  'orange-50': [0.980, 0.021, 75],
  'orange-100': [0.955, 0.050, 75],
  'orange-700': [0.540, 0.135, 42],
  'orange-900': [0.408, 0.094, 40],
  'yellow-50': [0.987, 0.026, 100],
  'yellow-100': [0.973, 0.060, 100],
  'yellow-600': [0.550, 0.132, 85],
  'yellow-800': [0.523, 0.095, 75],
  'yellow-900': [0.452, 0.085, 70],
  'purple-100': [0.954, 0.043, 308],
  'purple-700': [0.533, 0.175, 305],

  // Dark mode overrides
  'gray-100-dark': [0.967, 0.003, 264],
  'gray-400-dark': [0.650, 0.015, 261],
  'gray-500-dark': [0.600, 0.018, 264],
  'gray-900-dark': [0.140, 0.006, 264],
  'gray-950-dark': [0.130, 0.004, 264],
  'blue-600-dark': [0.585, 0.220, 262],
  'green-300-dark': [0.850, 0.120, 152],
  'red-300-dark': [0.780, 0.145, 22],
  'orange-300-dark': [0.880, 0.110, 68],
  'yellow-300-dark': [0.900, 0.125, 95],
};

// Calculate luminance for each color
const luminances = {};
for (const [name, [L, C, H]] of Object.entries(colors)) {
  const [r, g, b] = oklchToRgb(L, C, H);
  luminances[name] = relativeLuminance(r, g, b);
}

// Test pairs from the plan
const pairs = [
  // Light mode
  { text: 'gray-900', bg: 'white', name: 'text-primary on surface-primary', min: 4.5 },
  { text: 'gray-900', bg: 'gray-50', name: 'text-primary on surface-secondary', min: 4.5 },
  { text: 'gray-600', bg: 'white', name: 'text-secondary on surface-primary', min: 4.5 },
  { text: 'gray-600', bg: 'gray-50', name: 'text-secondary on surface-secondary', min: 4.5 },
  { text: 'gray-500', bg: 'white', name: 'text-tertiary on surface-primary', min: 4.5 },
  { text: 'gray-500', bg: 'gray-50', name: 'text-tertiary on surface-secondary', min: 4.5 },
  { text: 'white', bg: 'blue-600', name: 'text-inverse on brand-primary', min: 4.5 },
  { text: 'white', bg: 'gray-900', name: 'text-inverse on surface-inverse', min: 4.5 },
  { text: 'red-700', bg: 'red-50', name: 'danger-text on danger-bg', min: 4.5 },
  { text: 'green-800', bg: 'green-50', name: 'success-text on success-bg', min: 4.5 },
  { text: 'red-700', bg: 'gray-100', name: 'severity-critical-text on severity-critical-bg', min: 4.5 },
  { text: 'orange-700', bg: 'orange-100', name: 'severity-high-text on severity-high-bg', min: 4.5 },
  { text: 'yellow-600', bg: 'yellow-100', name: 'severity-medium-text on severity-medium-bg', min: 4.5 },
  { text: 'blue-700', bg: 'blue-100', name: 'severity-info-text on severity-info-bg (not in globals)', min: 4.5 },
  { text: 'gray-700', bg: 'gray-100', name: 'severity-none-text on severity-none-bg', min: 4.5 },
  { text: 'purple-700', bg: 'purple-100', name: 'category-text on category-bg', min: 4.5 },
  { text: 'blue-600', bg: 'white', name: 'brand-primary links on surface-primary', min: 4.5 },
  { text: 'gray-400', bg: 'white', name: 'text-muted on surface-primary (large text only)', min: 3.0 },

  // Dark mode
  { text: 'gray-100-dark', bg: 'gray-950-dark', name: '[DARK] text-primary on surface-primary', min: 4.5 },
  { text: 'gray-400-dark', bg: 'gray-950-dark', name: '[DARK] text-secondary on surface-primary', min: 4.5 },
  { text: 'gray-400-dark', bg: 'gray-900-dark', name: '[DARK] text-secondary on surface-secondary', min: 4.5 },
  { text: 'gray-500-dark', bg: 'gray-950-dark', name: '[DARK] text-tertiary on surface-primary', min: 4.5 },
  { text: 'gray-900-dark', bg: 'blue-600-dark', name: '[DARK] text-inverse on brand-primary', min: 4.5 },
  { text: 'red-300-dark', bg: 'red-950', name: '[DARK] danger-text on danger-bg', min: 4.5 },
  { text: 'green-300-dark', bg: 'green-950', name: '[DARK] success-text on success-bg', min: 4.5 },
  { text: 'red-300-dark', bg: 'red-900', name: '[DARK] severity-critical-text on severity-critical-bg', min: 4.5 },
  { text: 'orange-300-dark', bg: 'orange-900', name: '[DARK] severity-high-text on severity-high-bg', min: 4.5 },
  { text: 'yellow-300-dark', bg: 'yellow-900', name: '[DARK] severity-medium-text on severity-medium-bg', min: 4.5 },
];

console.log('WCAG AA Contrast Ratio Validation Report');
console.log('=========================================\n');

let passCount = 0;
let failCount = 0;
const failures = [];

for (const pair of pairs) {
  const textLum = luminances[pair.text];
  const bgLum = luminances[pair.bg];

  if (textLum === undefined || bgLum === undefined) {
    console.log(`⚠️  SKIP: ${pair.name} (missing color data)`);
    continue;
  }

  const ratio = contrastRatio(textLum, bgLum);
  const pass = ratio >= pair.min;

  if (pass) {
    console.log(`✅ PASS: ${pair.name}`);
    console.log(`   Ratio: ${ratio.toFixed(2)}:1 (min: ${pair.min}:1)`);
    passCount++;
  } else {
    console.log(`❌ FAIL: ${pair.name}`);
    console.log(`   Ratio: ${ratio.toFixed(2)}:1 (min: ${pair.min}:1)`);
    console.log(`   Colors: ${pair.text} (L=${luminances[pair.text].toFixed(3)}) on ${pair.bg} (L=${luminances[pair.bg].toFixed(3)})`);
    failCount++;
    failures.push({ ...pair, ratio, textLum, bgLum });
  }
  console.log('');
}

console.log('\n=========================================');
console.log(`Summary: ${passCount} passed, ${failCount} failed`);
console.log('=========================================\n');

if (failures.length > 0) {
  console.log('FAILURES REQUIRING FIXES:\n');
  for (const failure of failures) {
    console.log(`${failure.name}:`);
    console.log(`  Current ratio: ${failure.ratio.toFixed(2)}:1`);
    console.log(`  Required: ${failure.min}:1`);
    console.log(`  Gap: ${(failure.min - failure.ratio).toFixed(2)}`);

    // Calculate required luminance adjustment
    const targetRatio = failure.min + 0.1; // Add 0.1 buffer
    let requiredTextLum;

    if (failure.textLum > failure.bgLum) {
      // Text is lighter, need to increase contrast
      requiredTextLum = targetRatio * (failure.bgLum + 0.05) - 0.05;
    } else {
      // Text is darker, need to decrease luminance
      requiredTextLum = (failure.bgLum + 0.05) / targetRatio - 0.05;
    }

    console.log(`  Required text luminance: ${requiredTextLum.toFixed(3)} (current: ${failure.textLum.toFixed(3)})`);
    console.log('');
  }

  process.exit(1);
} else {
  console.log('✅ All contrast ratios pass WCAG AA requirements!');
  process.exit(0);
}
