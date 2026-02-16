import sharp from 'sharp'
import { writeFile } from 'node:fs/promises'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')
const input = join(root, 'public', 'logo.png')

function createIco(pngBuffers) {
  const count = pngBuffers.length
  const headerSize = 6
  const dirEntrySize = 16
  const dataOffset = headerSize + dirEntrySize * count

  // ICO header: reserved(2) + type(2) + count(2)
  const header = Buffer.alloc(headerSize)
  header.writeUInt16LE(0, 0)     // reserved
  header.writeUInt16LE(1, 2)     // type = ICO
  header.writeUInt16LE(count, 4) // image count

  const dirEntries = []
  let offset = dataOffset

  for (const png of pngBuffers) {
    // Parse dimensions from PNG header (width/height at bytes 16-23)
    const width = png.readUInt32BE(16)
    const height = png.readUInt32BE(18)

    const entry = Buffer.alloc(dirEntrySize)
    entry.writeUInt8(width >= 256 ? 0 : width, 0)
    entry.writeUInt8(height >= 256 ? 0 : height, 1)
    entry.writeUInt8(0, 2)       // color palette
    entry.writeUInt8(0, 3)       // reserved
    entry.writeUInt16LE(1, 4)    // color planes
    entry.writeUInt16LE(32, 6)   // bits per pixel
    entry.writeUInt32LE(png.length, 8)
    entry.writeUInt32LE(offset, 12)

    dirEntries.push(entry)
    offset += png.length
  }

  return Buffer.concat([header, ...dirEntries, ...pngBuffers])
}

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
  const ico = createIco([png32, png16])
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
