from transformers import AutoModelForSequenceClassification
from transformers import AutoTokenizer
import numpy as np
from scipy.special import softmax
import re
import spacy
import urllib
import csv

nlp = spacy.load("en_core_web_sm")



MODEL = "cardiffnlp/twitter-roberta-base-sentiment"
tokenizer = AutoTokenizer.from_pretrained(MODEL)
model = AutoModelForSequenceClassification.from_pretrained(MODEL)

task='sentiment'
labels=[]
mapping_link = f"https://raw.githubusercontent.com/cardiffnlp/tweeteval/main/datasets/{task}/mapping.txt"
with urllib.request.urlopen(mapping_link) as f:
    html = f.read().decode('utf-8').split("\n")
    csvreader = csv.reader(html, delimiter='\t')
labels = [row[1] for row in csvreader if len(row) > 1]


def preprocess_text(text):
    """
    Preprocess the input text using the provided spaCy model.

    Args:
        text (str): The input text to preprocess.
        nlp_model: The preloaded spaCy language model.

    Returns:
        str: The preprocessed text.
    """
    # Process the text with the spaCy model
    doc = nlp(text)
    processed_text = doc.text.strip()
    return processed_text

def get_sentiment(text) -> dict:
    try:
        text = str(text)
        text = preprocess_text(text)
    
    
        if len(text) == 0:
            return {
                "positive": 0.0,
                "neutral": 0.0,
                "negative": 0.0
            }
    
        encoded_input = tokenizer(text, return_tensors='pt')
        output = model(**encoded_input)
        scores = output[0][0].detach().numpy()
        scores = softmax(scores)

        ranking = np.argsort(scores)
        ranking = ranking[::-1]

        sentiment={}

        for i in range(scores.shape[0]):
            l = labels[ranking[i]]
            s = scores[ranking[i]]
            sentiment[l]=s
        
        return sentiment
        
    except:
        return {
            "positive": 0.0,
            "neutral": 0.0,
            "negative": 0.0
        }



