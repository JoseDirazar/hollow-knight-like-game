
from PIL import Image

# Configuración
input_path = "skeletonAttack-Sheet146x64.png"
output_path = "skeletonAttack-cropped.png"
frame_width = 146
frame_height = 64
y_offset = 5  # Desplazamiento hacia abajo

# Cargar spritesheet original
spritesheet = Image.open(input_path)
sheet_width, sheet_height = spritesheet.size

cols = sheet_width // frame_width
rows = sheet_height // frame_height

# Nueva hoja con mismo tamaño total
new_sheet = Image.new("RGBA", (sheet_width, sheet_height), (0, 0, 0, 0))

for row in range(rows):
    for col in range(cols):
        left = col * frame_width
        top = row * frame_height
        frame = spritesheet.crop((left, top, left + frame_width, top + frame_height))

        # Crear nuevo marco y pegar con desplazamiento hacia abajo
        new_frame = Image.new("RGBA", (frame_width, frame_height), (0, 0, 0, 0))
        new_frame.paste(frame, (0, y_offset))  # offset Y

        # Pegar en hoja final
        new_sheet.paste(new_frame, (left, top))

new_sheet.save(output_path)
print(f"Spritesheet corregido con offset guardado en: {output_path}")
