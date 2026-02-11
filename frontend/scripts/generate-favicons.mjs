import sharp from 'sharp'
import toIco from 'to-ico'
import { writeFile } from 'node:fs/promises'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')
const input = join(root, 'public', 'logo.png')

async function generateFavicons() {
  // Generate 16x16 and 32x32 PNGs for ICO
  const png16 = await sharp(input)
    .resize(16, 16, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer()

  const png32 = await sharp(input)
    .resize(32, 32, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer()

  // Create multi-resolution ICO (32x32 first for primary display)
  const ico = await toIco([png32, png16])
  await writeFile(join(root, 'app', 'favicon.ico'), ico)
  console.log('Generated: app/favicon.ico (16x16 + 32x32)')

  // Generate Apple touch icon (180x180)
  await sharp(input)
    .resize(180, 180, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(join(root, 'app', 'apple-icon.png'))
  console.log('Generated: app/apple-icon.png (180x180)')
}

generateFavicons().catch(err => {
  console.error('Favicon generation failed:', err)
  process.exit(1)
})
