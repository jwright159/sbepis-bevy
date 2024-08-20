from PIL import Image

# Load the image
image = Image.open('prototype_512x512_clear.png').convert('RGBA')

# Split the image into its respective channels
r, g, b, a = image.split()

# Create a new alpha channel using the red channel
new_alpha = r

# Create new RGB channels set to white
white = Image.new('L', image.size, 255)

# Merge the new RGB channels with the new alpha channel
new_image = Image.merge('RGBA', (white, white, white, new_alpha))

# Save the modified image
new_image.save('prototype_512x512_clear2.png')

